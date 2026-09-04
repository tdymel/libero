use dioxus::prelude::{Event, MouseData};

/// Whether this click landed on something interactive of its own - a link, a
/// button, a field - nested inside the nearest ancestor that matches
/// `boundary`. A handler on that ancestor then leaves the click to it: per
/// HTML a `<label>` does not forward such a click to its control, and a card
/// should not either.
///
/// It has to be asked of the platform: `MouseData` carries no target, so the
/// renderer's own event is the only way to one ([[codebase/dioxus-event-data]]).
///
/// **Only the wasm32 arm answers.** Everywhere else it is `false`, which keeps
/// the old behaviour: the whole of the boundary is one click target.
pub(crate) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    super::backend::nested_interactive(event, boundary)
}
