use dioxus::prelude::{Event, TransitionData};

use super::backend;

/// The CSS property whose transition just ended.
///
/// This is a platform question and not something a hook can answer on its own.
/// `TransitionData` keeps its `Box<dyn HasTransitionData>` private and does not
/// implement that public trait, so it exposes only `new` and `downcast` -
/// downcasting to the renderer's own event type is the only public route to
/// any transition field. See [[codebase/dioxus-event-data]] in the brain.
///
/// `None` where this build cannot read one, the house rule for an absent
/// capability. It means "cannot tell", never "some other property", so a
/// caller filtering on a property has to decide which way to fail.
pub(crate) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    backend::transition_property(event)
}
