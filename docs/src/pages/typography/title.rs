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
                Title { variant: "h1", "Title" }
                Text {
                    "A heading, h1 through h6 - "
                    Code { "component" }
                    " decouples the semantic tag from the visual size, for a11y heading order."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Variants" }
                Flex {
                    direction: "column",
                    gap: "8px",
                    Title { variant: "h1", "Heading one" }
                    Title { variant: "h2", "Heading two" }
                    Title { variant: "h3", "Heading three" }
                    Title { variant: "h4", "Heading four" }
                    Title { variant: "h5", "Heading five" }
                    Title { variant: "h6", "Heading six" }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Decoupled tag" }
                Text { "Sized like h1, but rendered as a p - doesn't affect the page's heading order." }
                Title { variant: "h1", component: "p", "Looks like h1, isn't one" }
            }
        }
    }
}
