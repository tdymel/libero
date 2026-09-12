use std::rc::Rc;

use dioxus::prelude::{Event, FocusData, MountedData, PointerData};

use super::{ElementApi, backend};

/// Gives a pressed drag handle the focus its cancelled pointerdown took away:
/// the outermost tab stop from the press's target up to `within`, unless focus
/// is already inside it. **Only the wasm32 arm acts**: nothing else carries a
/// target.
pub(crate) fn focus_pressed(event: &Event<PointerData>, within: &Rc<MountedData>) {
    backend::focus_pressed(event, within);
}

/// Focus moves a renderer makes without firing any focus event.
pub(crate) trait SilentFocusApi {
    /// Calls `callback` after such a move has landed, until the returned
    /// subscription is dropped.
    fn on_move(&self, callback: OnMove) -> Box<dyn SilentFocusSubscription>;
}

pub(crate) type OnMove = Box<dyn Fn(&dyn FocusMove)>;

/// One silent move, the `focusout`/`focusin` pair it stands in for.
pub(crate) trait FocusMove {
    /// Whether focus was on `element` or inside it before the move.
    fn was_in(&self, element: &Rc<MountedData>) -> bool;
    /// Whether it is now.
    fn is_in(&self, element: &Rc<MountedData>) -> bool;
}

/// Dropping it stops the callbacks.
pub(crate) trait SilentFocusSubscription {}

/// `None` where every focus move fires its events (the web). Blitz fires none
/// for Tab, Shift+Tab and libero's own `focus()`/`blur()`, and reports those.
pub(crate) fn silent_focus() -> Option<&'static dyn SilentFocusApi> {
    backend::silent_focus()
}

/// Whether the blur being handled comes from a press that cancelled its
/// `mousedown`, where the web keeps focus put. Blitz moves it anyway and
/// libero moves it back, so a field closing on blur ignores this one.
pub(crate) fn press_kept_focus() -> bool {
    backend::press_kept_focus()
}

/// Whether the element this `focusin` landed on matches `:focus-visible`: the
/// browser's own call on keyboard vs pointer focus, script focus after a press
/// included. `None` off the web; the caller keeps its own press heuristic there.
pub(crate) fn focus_visible(event: &Event<FocusData>) -> Option<bool> {
    backend::focus_visible(event)
}

/// For a `focusin` on the nearest ancestor matching `boundary`: the element
/// focus left to get here, `Some(None)` when it came from `<body>`.
///
/// `None` when it moved within `boundary`, or where this build cannot tell:
/// **only the wasm32 arm answers** (`FocusData` carries no `relatedTarget`).
pub(crate) fn focus_entered_from(
    event: &Event<FocusData>,
    boundary: &str,
) -> Option<Option<Box<dyn ElementApi>>> {
    backend::focus_entered_from(event, boundary)
}
