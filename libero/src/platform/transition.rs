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
/// The web and the desktop/Android WebView answer - a WebView runs a real
/// browser engine and fires `transitionend` per property. Blitz and a server
/// answer `None`, and [`use_presence`](crate::hooks::use_presence) takes an
/// unreadable property as the exit it waits for.
pub(crate) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    backend::transition_property(event)
}

/// The WebView arm, against the payload dioxus-desktop actually builds: its
/// converter turns the IPC message into a `SerializedTransitionData` and
/// wraps that, so the downcast has to find it there.
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
