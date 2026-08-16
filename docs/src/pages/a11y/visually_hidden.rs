use dioxus::prelude::*;
use libero::components::{Anchor, Flex, Text, Title, VisuallyHidden};

#[component]
pub fn VisuallyHiddenPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Visually Hidden" }
                Text {
                    "Content available to screen readers but removed from sighted layout - "
                    "e.g. extra context for a link that's ambiguous out of context."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Example" }
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
