use dioxus::prelude::*;
use libero::components::{Anchor, Flex, Text, Title, VisuallyHidden};

#[component]
pub fn VisuallyHiddenPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Visually Hidden" }
                Text {
                    "Content available to screen readers but removed from sighted layout - "
                    "e.g. extra context for a link that's ambiguous out of context."
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Example" }
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
