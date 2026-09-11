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
