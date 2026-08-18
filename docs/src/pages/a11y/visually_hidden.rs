use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Anchor, Text, VisuallyHidden};

#[component]
pub fn VisuallyHiddenPage() -> Element {
    rsx! {
        DocPage {
            title: "Visually Hidden",
            lead: rsx! {
                Text {
                    "Content available to screen readers but removed from sighted layout - "
                    "e.g. extra context for a link that's ambiguous out of context."
                }
            },
            DocSection {
                title: "Example",
                Text {
                    Anchor {
                        to: "https://example.com",
                        "Read more"
                        VisuallyHidden { " about focus management" }
                    }
                }
            }
        }
    }
}
