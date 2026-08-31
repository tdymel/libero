use dioxus::prelude::{Event, TransitionData};

use super::backend;

/// The CSS property whose transition just ended, or `None` where this build
/// cannot tell.
///
/// It has to be asked of the platform: `TransitionData` keeps its inner box
/// private and does not implement the public `HasTransitionData`, so
/// downcasting to the renderer's own event type is the only way to any
/// transition field ([[codebase/dioxus-event-data]]).
///
/// **Only the wasm32 arm answers.** The desktop and Android WebView backends
/// run a browser engine and do fire `transitionend` per property, so `None`
/// there is a known wrong answer, not an absent capability - see [[todos]].
pub(crate) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    backend::transition_property(event)
}
