use dioxus::prelude::*;

use super::{
    CAROUSEL_HEIGHT, CarouselPart,
    state::{CarouselView, Nav},
};
use crate::{
    components::{
        common::{
            Input, Orientation, Part, States, has_shortcut_modifier, inset_focus_ring_sx, states,
        },
        layout::{ScrollArea, ScrollAreaBase, ScrollPositionEvent, scroll_area_base},
    },
    hooks::Drag,
    platform::{arrow_target, key_taken, logical_key, snaps_scroll, typing_target, when_free},
    sx::{REDUCED_MOTION, StaticSx, sx},
    theme::CAROUSEL_GAP,
};

/// Merged onto `ScrollArea`'s own; the indicators replace the hidden scrollbar.
pub(super) static CAROUSEL_TRACK_SX: StaticSx = StaticSx::new(|| {
    scroll_area_base(
        sx().display("flex")
            // `overridable`, not `value`: the props write the `-override` twin.
            .gap(CAROUSEL_GAP.overridable())
            .when("horizontal", sx().flex_direction("row"))
            .when("vertical", sx().flex_direction("column"))
            .when("horizontal", sx().scroll_snap_type("x mandatory"))
            .when("vertical", sx().scroll_snap_type("y mandatory"))
            // Keeps a fast flick from skipping past a slide.
            .scroll_snap_stop("always")
            .overscroll_behavior_x("contain")
            .overscroll_behavior_y("contain")
            .height(CAROUSEL_HEIGHT.value_or("auto"))
            .scroll_behavior("smooth")
            // Chrome and Safari keep smooth scrolling under reduced motion.
            .media(REDUCED_MOTION, sx().scroll_behavior("auto"))
            // A drag follows the pointer: smoothing lags and a snap pulls it back.
            // After the orientation arms, which it beats at equal specificity.
            .when(
                "dragging",
                sx().scroll_behavior("auto").scroll_snap_type("none"),
            )
            .when("seam", sx().scroll_behavior("auto"))
            // Inset: the `overflow: hidden` viewport would clip an outset ring.
            .focus_visible(inset_focus_ring_sx("-2px")),
    )
});

/// The scroll container and tab stop: the strip, scroll listeners, arrow keys and drag.
pub(super) fn carousel_track(
    view: CarouselView,
    aria_label: String,
    draggable: bool,
    drag: Drag,
    body: Vec<Element>,
) -> Element {
    let CarouselView {
        setup,
        state,
        track_id,
        status_id,
        ..
    } = view;
    let nav = state.nav;
    let orientation = setup.orientation;
    let empty = setup.count == 0;
    let dragging = state.dragging;
    let mut current = nav.current;
    let mut settled = nav.settled;
    let mut seam = nav.seam;

    let mut onscroll = move |x: f64, y: f64| {
        let next = nav.real_for(nav.raw_at(x, y));
        if next != *current.peek() {
            current.set(next);
        }
    };

    // Only a settled scroll moves `settled`, which the live region and `onindexchange` read.
    let mut onscrollend = move |x: f64, y: f64| {
        // A drag's per-move scrolls end too, but are no settle: they jumped the seam.
        if *dragging.peek() {
            return;
        }
        let raw = nav.raw_at(x, y);
        // No scroll snap natively: go to the nearest slide, as a released drag does.
        if !snaps_scroll() {
            let real = nav.real_for(raw);
            when_free(move || nav.go_to(real));
            return;
        }
        let next = nav.real_for(raw);
        current.set(next);
        if next != *settled.peek() {
            settled.set(next);
            if let Some(handler) = &nav.onindexchange {
                handler.call(next);
            }
        }
        // Settled on a clone: jump to the real slide; that landing crosses nothing.
        match nav.is_clone(raw) {
            true => nav.cross_seam(),
            false => {
                if *seam.peek() {
                    seam.set(false);
                }
            }
        }
    };

    // On the track, so the sibling controls' keys never reach it. The guards skip
    // keys slide content needs: taken presses, text entry, native arrow inputs.
    let onkeydown = move |event: Event<KeyboardData>| {
        if key_taken(&event)
            || typing_target(&event)
            || arrow_target(&event)
            || has_shortcut_modifier(&event)
        {
            return;
        }
        let Some(target) = track_key_target(logical_key(&event), orientation, nav) else {
            return;
        };
        // Or the native scroll runs too and lands between two snap points.
        event.prevent_default();
        match target {
            TrackKey::Step(forward) => nav.step(forward),
            TrackKey::To(index) => nav.go_to(index),
        }
    };

    let track_states: Input<States> = states()
        .with(orientation.state_name(), true)
        .with("dragging", dragging())
        // The seam jump has to be instant, or the strip visibly rewinds.
        .with("seam", seam())
        .into();

    rsx! {
        ScrollArea {
            handle: setup.track,
            scrollbars: match orientation {
                Orientation::Horizontal => "horizontal",
                Orientation::Vertical => "vertical",
            },
            scrollbar_visibility: "hidden",
            // The tab stop, unlike `ScrollArea`'s default; not when empty.
            focusable: !empty && !setup.quiet,
            framework_sx: ScrollAreaBase(&CAROUSEL_TRACK_SX),
            states: track_states,
            "data-slot": CarouselPart::Track.slot(),
            id: track_id(),
            // Named itself: landing inside does not reliably re-announce the region.
            role: (!empty).then_some("group"),
            aria_label: (!empty).then(|| aria_label.clone()),
            aria_describedby: (!empty && !setup.quiet).then_some(status_id()),
            onscroll: move |event: ScrollPositionEvent| match event {
                ScrollPositionEvent::Start(x, y) | ScrollPositionEvent::Change(x, y) => {
                    onscroll(x, y)
                }
                ScrollPositionEvent::End(x, y) => onscrollend(x, y),
            },
            onkeydown,
            // Mouse only: a touch is already scrolling the track natively, and
            // dragging it too would move it twice.
            onpointerdown: move |event: Event<PointerData>| {
                if draggable && event.data().pointer_type() == "mouse" {
                    drag.onpointerdown.call(event);
                }
            },
            onpointermove: drag.onpointermove,
            onpointerup: drag.onpointerup,
            onpointercancel: drag.onpointercancel,
            {body.into_iter()}
        }
    }
}

/// What a track key does: a step, which wraps a looping strip, or a jump.
enum TrackKey {
    Step(bool),
    To(usize),
}

/// What an arrow, `Home` or `End` on the track does; `None` for a key the
/// carousel does not act on. A plain strip ends at the reachable window.
fn track_key_target(key: Key, orientation: Orientation, nav: Nav) -> Option<TrackKey> {
    let (previous, next) = match orientation {
        Orientation::Horizontal => (Key::ArrowLeft, Key::ArrowRight),
        Orientation::Vertical => (Key::ArrowUp, Key::ArrowDown),
    };
    let wraps = nav.clones > 0;
    Some(match key {
        key if key == previous => TrackKey::Step(false),
        key if key == next => TrackKey::Step(true),
        Key::Home => TrackKey::To(match wraps {
            true => 0,
            false => nav.first,
        }),
        Key::End => TrackKey::To(match wraps {
            true => nav.count.saturating_sub(1),
            false => nav.last,
        }),
        _ => return None,
    })
}
