use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Text, Title};

#[component]
pub fn TitlePage() -> Element {
    rsx! {
        DocPage {
            title: "Title",
            lead: rsx! {
                Text {
                    "A heading, h1 through h6 - "
                    Code { "component" }
                    " decouples the semantic tag from the visual size, for a11y heading order."
                }
            },
            DocSection {
                title: "Variants",
                Flex {
                    direction: "column",
                    gap: "sm",
                    Title { size: "xxl", "Heading one" }
                    Title { size: "xl", "Heading two" }
                    Title { size: "lg", "Heading three" }
                    Title { size: "md", "Heading four" }
                    Title { size: "sm", "Heading five" }
                    Title { size: "xs", "Heading six" }
                }
            }
            DocSection {
                title: "Decoupled tag",
                Text { "Sized like h1, but rendered as a p - doesn't affect the page's heading order." }
                Title { size: "xxl", component: "p", "Looks like h1, isn't one" }
            }
        }
    }
}
