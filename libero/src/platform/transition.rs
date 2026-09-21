use dioxus::prelude::{Event, TransitionData};

use super::backend;

/// The CSS property whose transition just ended, `None` where unreadable: only a
/// downcast reaches it ([[codebase/dioxus-event-data]]). Web and WebView answer.
///
/// Blitz sends no `transitionend`, so its exits end on
/// [`use_presence`](crate::hooks::use_presence)'s timer.
pub(crate) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    backend::transition_property(event)
}

/// The WebView arm, against dioxus-desktop's real payload: a wrapped
/// `SerializedTransitionData`.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::rc::Rc;

    use dioxus::html::{HasTransitionData, SerializedTransitionData};

    use super::*;

    struct Ended(&'static str);

    impl HasTransitionData for Ended {
        fn property_name(&self) -> String {
            self.0.to_string()
        }
        fn pseudo_element(&self) -> String {
            String::new()
        }
        fn elapsed_time(&self) -> f32 {
            0.6
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    fn event(data: TransitionData) -> Event<TransitionData> {
        Event::new(Rc::new(data), true)
    }

    #[test]
    fn a_webview_payload_names_its_property() {
        let serialized = SerializedTransitionData::from(&TransitionData::new(Ended("opacity")));
        let event = event(TransitionData::new(serialized));

        assert_eq!(transition_property(&event).as_deref(), Some("opacity"));
    }

    /// Blitz's payload, or any other: unreadable, not guessed at.
    #[test]
    fn any_other_payload_answers_none() {
        let event = event(TransitionData::new(Ended("opacity")));

        assert_eq!(transition_property(&event), None);
    }
}
