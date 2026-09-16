use dioxus::prelude::*;

use crate::components::VisuallyHidden;

/// A polite live region for one-off messages: a refusal, an error on Enter.
/// Render it unconditionally, since a region inserted with its text is not
/// announced.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Announcer {
    /// Counted, so a message equal to the last one still lands as a new node.
    said: Signal<Option<(u64, String)>>,
}

impl Announcer {
    pub(crate) fn say(mut self, message: String) {
        let count = self.said.peek().as_ref().map_or(0, |(count, _)| count + 1);
        self.said.set(Some((count, message)));
    }

    /// Empties the region, so a stale message is not read on a later visit.
    pub(crate) fn clear(mut self) {
        if self.said.peek().is_some() {
            self.said.set(None);
        }
    }

    pub(crate) fn render(self) -> Element {
        let said = self.said.read().clone();
        rsx! {
            VisuallyHidden { role: "status",
                for (count, message) in said {
                    span { key: "{count}", "{message}" }
                }
            }
        }
    }
}

pub(crate) fn use_announcer() -> Announcer {
    Announcer {
        said: use_signal(|| None),
    }
}
