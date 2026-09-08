use dioxus::prelude::{Event, FocusData};

use super::{ElementApi, backend};

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
