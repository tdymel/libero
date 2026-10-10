use std::cmp::Ordering;

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use super::{
    CarouselPart, numbered,
    state::{CarouselView, snap_position},
};
use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::ActionIcon,
        common::{Glyph, Input, Orientation, Part, States, focus_ring_sx, shadow_sx, states},
        layout::Box,
    },
    context::IconSlot,
    localization::{CarouselLabels, fill},
    sx::{StaticSx, Sx, sx},
    theme::{
        CAROUSEL_CONTROL_BACKGROUND, CAROUSEL_CONTROL_COLOR, CAROUSEL_CONTROL_HOVER_BACKGROUND,
        CAROUSEL_CONTROL_SIZE, CAROUSEL_CONTROLS_OFFSET, CssVar, FOCUS_RING_HALO, NamedColorCss,
        Size, SizeCss,
    },
};

static CAROUSEL_CONTROLS_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .display("flex")
        .justify_content("space-between")
        .align_items("center")
        // Only the buttons take the pointer; the strip underneath keeps it.
        .pointer_events("none")
        .when(
            "horizontal",
            sx().top("0")
                .bottom("0")
                .left(CAROUSEL_CONTROLS_OFFSET.value())
                .right(CAROUSEL_CONTROLS_OFFSET.value()),
        )
        .when(
            "vertical",
            sx().left("0")
                .right("0")
                .top(CAROUSEL_CONTROLS_OFFSET.value())
                .bottom(CAROUSEL_CONTROLS_OFFSET.value())
                .flex_direction("column"),
        )
});

/// Every control's fill and glyph. A var is opaque to `background()`, so the
/// focus contrast and halo are declared by hand.
fn control_colors_sx() -> Sx {
    sx().background(CAROUSEL_CONTROL_BACKGROUND.value())
        .color(CAROUSEL_CONTROL_COLOR.value())
        .var(
            CssVar::Owned(NamedColorCss::FOCUS_CONTRAST.name().to_string()),
            CAROUSEL_CONTROL_COLOR.value(),
        )
        .var(FOCUS_RING_HALO, CAROUSEL_CONTROL_BACKGROUND.value())
}

pub(super) static CAROUSEL_CONTROL_SX: StaticSx = StaticSx::new(|| {
    sx().pointer_events("auto")
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .width(CAROUSEL_CONTROL_SIZE.value())
        .height(CAROUSEL_CONTROL_SIZE.value())
        .padding("0")
        .border_width("0")
        .border_radius("50%")
        .and(control_colors_sx())
        .and(shadow_sx(SizeCss::SHADOW.value(Size::Sm)))
        .cursor("pointer")
        .selector("& > svg", sx().width("60%").height("60%"))
        // The row runs right to left, so the arrows point the other way.
        .when(
            "horizontal",
            sx().rtl(sx().selector("& > svg", sx().transform("scaleX(-1)"))),
        )
        .hover(sx().background(CAROUSEL_CONTROL_HOVER_BACKGROUND.value()))
        // Disabled keeps its tab stop, so focus is never dropped at either end.
        .when(
            "disabled",
            sx().opacity("0.4")
                .cursor("default")
                .background(CAROUSEL_CONTROL_BACKGROUND.value()),
        )
        .focus_visible(focus_ring_sx())
});

pub(super) static CAROUSEL_PAUSE_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        // First in the DOM for Tab order, so the viewport would paint over it.
        .z_index("1")
        .bottom(CAROUSEL_CONTROLS_OFFSET.value())
        .right(CAROUSEL_CONTROLS_OFFSET.value())
        .display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .width(CAROUSEL_CONTROL_SIZE.value())
        .height(CAROUSEL_CONTROL_SIZE.value())
        .padding("0")
        .border_width("0")
        .border_radius("50%")
        .and(control_colors_sx())
        .and(shadow_sx(SizeCss::SHADOW.value(Size::Sm)))
        .cursor("pointer")
        .selector("& > svg", sx().width("55%").height("55%"))
        // Beside Next's column: on a short strip it covered Next.
        .when(
            "beside-next",
            sx().right(format!(
                "calc(2 * {} + {})",
                CAROUSEL_CONTROLS_OFFSET.value(),
                CAROUSEL_CONTROL_SIZE.value()
            )),
        )
        .focus_visible(focus_ring_sx())
});

/// The previous/next pair, over the viewport. A looping carousel has no ends,
/// so its controls never disable. Its own scope, as the status and the dots:
/// only these read `current`/`settled`, so a move skips the track.
#[component]
pub(super) fn CarouselControls(view: CarouselView) -> Element {
    let CarouselView { setup, state, .. } = view;
    let current = state.nav.current;
    let looping = state.nav.clones > 0;
    let (first, last) = (setup.first, setup.last);
    // Memos the strip hands down unread, so a move redraws only the button
    // whose end flipped.
    let at_start = use_memo(use_reactive!(
        |looping, first| !looping && current() <= first
    ));
    let at_end = use_memo(use_reactive!(|looping, last| !looping && current() >= last));
    let strip_states: Input<States> = states().with(setup.orientation.state_name(), true).into();

    rsx! {
        Box {
            framework_sx: &CAROUSEL_CONTROLS_SX,
            states: strip_states,
            "data-slot": CarouselPart::Controls.slot(),
            CarouselControl { view, forward: false, disabled: at_start }
            CarouselControl { view, forward: true, disabled: at_end }
        }
    }
}

/// One control. Focusable when disabled at its end, so it keeps its tab stop.
#[component]
fn CarouselControl(view: CarouselView, forward: bool, disabled: Memo<bool>) -> Element {
    let CarouselView {
        setup,
        state,
        labels,
        track_id,
        ..
    } = view;
    let nav = state.nav;
    let orientation = setup.orientation;
    let control_states: Input<States> = states().with(orientation.state_name(), true).into();

    rsx! {
        ActionIcon {
            sx: &CAROUSEL_CONTROL_SX,
            states: control_states,
            "data-slot": CarouselPart::Control.slot(),
            aria_controls: track_id(),
            disabled: disabled(),
            focusable_when_disabled: true,
            aria_label: if forward { labels.next } else { labels.previous },
            onclick: move |_| nav.step(forward),
            {
                let (slot, icon) = match (orientation, forward) {
                    (Orientation::Horizontal, false) => {
                        (IconSlot::ChevronLeft, lucide::chevron_left::outlined)
                    }
                    (Orientation::Vertical, false) => {
                        (IconSlot::ChevronUp, lucide::chevron_up::outlined)
                    }
                    (Orientation::Horizontal, true) => {
                        (IconSlot::ChevronRight, lucide::chevron_right::outlined)
                    }
                    (Orientation::Vertical, true) => {
                        (IconSlot::ChevronDown, lucide::chevron_down::outlined)
                    }
                };
                rsx! { Glyph { slot, icon } }
            }
        }
    }
}

/// The slides showing as a range; a looping window across the seam says it wraps.
pub(super) fn showing_status(
    labels: &CarouselLabels,
    (from, to): (usize, usize),
    count: usize,
) -> String {
    match from.cmp(&to) {
        Ordering::Equal => numbered(labels.status, from, count),
        Ordering::Greater => fill(
            labels.status_wrap,
            &[("from", &(from + 1)), ("to", &(to + 1)), ("n", &count)],
        ),
        Ordering::Less => fill(
            labels.status_range,
            &[("from", &(from + 1)), ("to", &(to + 1)), ("n", &count)],
        ),
    }
}

/// The live region: where the strip settled, out of the places it can rest.
#[component]
pub(super) fn CarouselStatus(view: CarouselView) -> Element {
    let CarouselView {
        setup,
        state,
        labels,
        status_id,
        ..
    } = view;
    let nav = state.nav;
    let settled = nav.settled;
    let status = match nav.per_view > 1.0 {
        // Several up, the slides showing are named, not the resting position (todo 550).
        true => showing_status(labels, nav.showing(settled()), setup.count),
        false => {
            let (position, positions) = snap_position(
                settled(),
                setup.count,
                setup.first,
                setup.last,
                nav.clones > 0,
            );
            numbered(labels.status, position, positions)
        }
    };

    rsx! {
        VisuallyHidden {
            id: status_id(),
            role: "status",
            // Off while autoplay rotates, `polite` once it stops (WCAG 2.2.2).
            aria_live: if state.running { "off" } else { "polite" },
            aria_atomic: "true",
            "{status}"
        }
    }
}

/// The autoplay toggle. Its name stays the same whether the slideshow runs or
/// not: `aria-pressed` carries the state.
pub(super) fn carousel_pause_button(view: CarouselView, controls: bool) -> Element {
    let mut paused = view.state.paused;
    let labels = view.labels;
    let (focused, mut pressing) = (view.state.focused, view.state.pressing);
    let mut released = view.state.released;
    let beside_next = controls && view.setup.orientation == Orientation::Horizontal;
    let pause_states: Input<States> = states().with("beside-next", beside_next).into();

    rsx! {
        Box {
            component: "button",
            r#type: "button",
            framework_sx: &CAROUSEL_PAUSE_SX,
            states: pause_states,
            "data-slot": CarouselPart::Pause.slot(),
            aria_label: labels.pause,
            aria_pressed: paused().to_string(),
            // The focus this press brings in would stop rotation before the
            // click toggles it back on.
            onpointerdown: move |_| {
                if !*focused.peek() {
                    pressing.set(true);
                }
            },
            // Focus that never came with the press (Safari, iOS): the next entry is a real one (todo 2359).
            onpointercancel: move |_| {
                if !*focused.peek() {
                    pressing.set(false);
                }
            },
            onclick: move |_| {
                if !*focused.peek() {
                    released.set(true);
                }
                paused.toggle();
            },
            if paused() {
                Glyph { slot: IconSlot::Play, icon: lucide::play::outlined }
            } else {
                Glyph { slot: IconSlot::Pause, icon: lucide::pause::outlined }
            }
        }
    }
}
