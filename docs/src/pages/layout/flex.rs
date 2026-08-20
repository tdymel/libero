use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Divider, Flex, Text},
    sx::sx,
};

#[component]
pub fn FlexPage() -> Element {
    rsx! {
        DocPage {
            title: "Flex",
            lead: rsx! {
                Text { "A flexbox container - direction, gap, align, justify and wrap, all theme-aware." }
            },
            DocSection {
                title: "Row with gap",
                Flex {
                    direction: "row",
                    gap: "md",
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Three" }
                }
            }
            DocSection {
                title: "Column",
                Flex {
                    direction: "column",
                    gap: "sm",
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
                    Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
                }
            }
            DocSection {
                title: "Divider",
                Text { "Rendered between each child - not before the first or after the last." }
                Text { "Needs the `dioxus-fork` build; upstream dioxus main merges children into one node, so no divider appears." }
                Flex {
                    direction: "column",
                    gap: "sm",
                    divider: rsx! { Divider {} },
                    Text { "First section" }
                    Text { "Second section" }
                    Text { "Third section" }
                }
            }
        }
    }
}
