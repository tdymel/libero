use std::rc::Rc;

use dioxus::prelude::{Callback, Event, MountedData, MouseData, PointerData};

use super::ElementApi;

/// Whether this click hit something interactive (link, button, field) nested in
/// the nearest `boundary`, which then leaves the click to it, as a `<label>` does.
///
/// Web and Blitz answer ([[codebase/dioxus-event-data]]); elsewhere `false`, the
/// whole boundary one target.
pub(crate) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    super::backend::nested_interactive(event, boundary)
}

/// Whether [`nested_interactive`] can answer. The Android WebView cannot: it
/// reads no click target (958).
pub(crate) fn reads_click_targets() -> bool {
    cfg!(any(target_arch = "wasm32", feature = "native"))
}

/// Whether an inline box in a padded block takes the pointer. Blitz's hit test
/// reaches one only when positioned with a positive `z-index` (889).
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

/// The control a press on a frame's padding belongs to: the first tab stop in
/// `boundary`'s child that is no `[data-slot]` but `control`, nor `[data-ring]`.
///
/// `None` when the press hit that control or anything interactive, and off wasm32.
pub(crate) fn padding_press(
    event: &Event<MouseData>,
    boundary: &str,
) -> Option<Box<dyn ElementApi>> {
    super::backend::padding_press(event, boundary)
}
