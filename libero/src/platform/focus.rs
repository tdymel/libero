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
