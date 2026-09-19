use std::rc::Rc;

use dioxus::prelude::{Callback, Event, MountedData, MouseData, PointerData};

use super::ElementApi;

/// Whether this click landed on something interactive of its own - a link, a
/// button, a field - nested inside the nearest ancestor that matches
/// `boundary`. A handler on that ancestor then leaves the click to it: per
/// HTML a `<label>` does not forward such a click to its control, and a card
/// should not either.
///
/// It has to be asked of the platform: `MouseData` carries no target, so the
/// renderer's own event is the only way to one ([[codebase/dioxus-event-data]]).
///
/// **The web and Blitz answer**; Blitz from the node its pointer press hit,
/// inside `LiberoProvider`. Everywhere else it is `false`, which keeps the old
/// behaviour: the whole of the boundary is one click target.
pub(crate) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    super::backend::nested_interactive(event, boundary)
}

/// Whether an inline box in a block with padding takes the pointer. Blitz's hit
/// test never reaches one there unless it is positioned with a positive
/// `z-index`, which it enters from the stacking context instead (todo 889).
pub(crate) fn hits_inline_boxes() -> bool {
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"))
}

/// Where `set_pointer_capture` failed, hands `capture`'s drag the moves and the
/// release of `event`'s pointer that land outside it, until that release.
/// Only Blitz does, from `LiberoProvider`'s wrapper; elsewhere a no-op.
pub(crate) fn follow_pointer(
    event: &Event<PointerData>,
    capture: &Rc<MountedData>,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
) {
    super::backend::follow_pointer(event, capture, onmove, onup)
}

/// Double presses where no `dblclick` follows a cancelled `pointerdown`: Blitz
/// counts clicks in that default action, so a drag handle never gets one.
#[derive(Clone, Copy, Default)]
pub(crate) struct DoublePress {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    last: Option<(std::time::Instant, f64, f64)>,
}

impl DoublePress {
    /// Whether this press at `(x, y)` makes a double press, as Blitz counts one:
    /// within 500ms and 2px of the last. Always `false` where `dblclick` works.
    pub(crate) fn press(&mut self, x: f64, y: f64) -> bool {
        #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
        {
            use std::time::{Duration, Instant};
            let now = Instant::now();
            let double = self.last.is_some_and(|(at, last_x, last_y)| {
                now - at < Duration::from_millis(500)
                    && (x - last_x).abs() <= 2.0
                    && (y - last_y).abs() <= 2.0
            });
            // A third press starts over, as a third click is no second double.
            self.last = (!double).then_some((now, x, y));
            double
        }
        #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
        {
            let _ = (x, y);
            false
        }
    }
}

/// The control a press on a frame's own padding belongs to: the first tab stop
/// in the child of the nearest `boundary` that is neither a `[data-slot]` nor
/// its `[data-ring]`. `None` when the press landed in that control, or on
/// anything interactive of its own - a slot's button keeps its press.
///
/// **Only the wasm32 arm answers**, as for [`nested_interactive`]. Elsewhere
/// the padding stays outside the press target.
pub(crate) fn padding_press(
    event: &Event<MouseData>,
    boundary: &str,
) -> Option<Box<dyn ElementApi>> {
    super::backend::padding_press(event, boundary)
}
