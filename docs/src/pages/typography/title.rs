use dioxus::prelude::*;
use libero::components::{Code, Flex, Text, Title};

#[component]
pub fn TitlePage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { size: "xxl", "Title" }
                Text {
                    "A heading, h1 through h6 - "
                    Code { "component" }
                    " decouples the semantic tag from the visual size, for a11y heading order."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Variants" }
                Flex {
                    direction: "column",
                    gap: "8px",
                    Title { size: "xxl", "Heading one" }
                    Title { size: "xl", "Heading two" }
                    Title { size: "lg", "Heading three" }
                    Title { size: "md", "Heading four" }
                    Title { size: "sm", "Heading five" }
                    Title { size: "xs", "Heading six" }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Decoupled tag" }
                Text { "Sized like h1, but rendered as a p - doesn't affect the page's heading order." }
                Title { size: "xxl", component: "p", "Looks like h1, isn't one" }
            }
        }
    }
}
