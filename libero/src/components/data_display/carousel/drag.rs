use dioxus::prelude::*;

use super::state::{CarouselSetup, CarouselState, Nav};
use crate::{
    components::{
        common::Orientation,
        layout::{inline_x, physical_x},
    },
    hooks::{Drag, DragMove, DragOptions, DragStart, use_distance_drag},
    platform::{ElementApi, snaps_scroll, when_free},
};

/// Mouse drag-to-scroll over the track. No `drag_handle_sx()`: its
/// `touch-action: none` would kill the native touch swipe.
pub(super) fn use_carousel_drag(
    setup: CarouselSetup,
    state: CarouselState,
    draggable: bool,
) -> Drag {
    let track = setup.track;
    let orientation = setup.orientation;
    let nav = state.nav;
    let mut dragging = state.dragging;
    let mut drag_origin = state.drag_origin;
    // Under RTL a drag to the right heads for the end, as in `Scroller`.
    let mut rtl = use_hook(|| CopyValue::new(false));

    // Only once it moves, so a click on a slide's link or button stays one (todo 2358).
    use_distance_drag(DragOptions {
        capture: track.element,
        onstart: Callback::new(move |start: DragStart| {
            if !draggable {
                start.cancel.call(());
                return;
            }
            dragging.set(true);
            // Unknown until the read lands: the move that starts the drag comes first.
            drag_origin.set(f64::NAN);
            let offset = track.element.scroll_offset();
            rtl.set(track.element.is_rtl());
            spawn(async move {
                if let Ok((x, y)) = offset.await {
                    drag_origin.set(match orientation {
                        Orientation::Horizontal => inline_x(x),
                        Orientation::Vertical => y,
                    });
                }
            });
        }),
        onmove: Callback::new(move |moved: DragMove| {
            if drag_origin().is_nan() {
                return;
            }
            let delta = moved.delta();
            let target = match orientation {
                Orientation::Horizontal => drag_origin() - physical_x(delta.x, rtl()),
                Orientation::Vertical => drag_origin() - delta.y,
            }
            .max(0.0);
            match orientation {
                Orientation::Horizontal => track.scroll_to(target, 0.0),
                Orientation::Vertical => track.scroll_to(0.0, target),
            }
        }),
        // Releasing hands the strip back to the browser, which snaps and
        // fires `onscrollend`. Blitz does neither: the nearest slide is gone to.
        onend: Callback::new(move |()| {
            dragging.set(false);
            if !snaps_scroll() {
                settle_nearest(nav);
            }
        }),
    })
}

/// Goes to the slide nearest the track's scroll offset.
fn settle_nearest(nav: Nav) {
    let element = nav.track.element;
    let (offset, content, view) = (
        element.scroll_offset(),
        element.scroll_size(),
        element.dimensions(),
    );
    spawn(async move {
        let (Ok((x, y)), Ok(content), Ok(view)) = (offset.await, content.await, view.await) else {
            return;
        };
        let percent = |at: f64, range: f64| match range > 0.0 {
            true => at / range * 100.0,
            false => 0.0,
        };
        let raw = nav.raw_at(
            percent(inline_x(x), content.width - view.width),
            percent(y, content.height - view.height),
        );
        // A move reads the track, which a task finds borrowed natively.
        when_free(move || nav.go_to(nav.real_for(raw)));
    });
}
