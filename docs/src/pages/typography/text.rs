use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Flex, Text},
    sx::sx,
};

#[component]
pub fn TextPage() -> Element {
    rsx! {
        DocPage {
            title: "Text",
            lead: rsx! {
                Text { "Body copy - renders a p by default, sized via the theme's text scale." }
            },
            DocSection {
                title: "Sizes",
                Flex {
                    direction: "column",
                    gap: "sm",
                    Text { size: "xs", "Extra small" }
                    Text { size: "sm", "Small" }
                    Text { size: "md", "Medium (default)" }
                    Text { size: "lg", "Large" }
                    Text { size: "xl", "Extra large" }
                }
            }
            DocSection {
                title: "As a span",
                Text {
                    "Inline text with "
                    Text { component: "span", sx: sx().font_weight("700"), "bold inline text" }
                    " in the middle."
                }
            }
        }
    }
}
