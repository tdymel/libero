use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Divider, Flex, Text},
    sx::sx,
};

#[component]
pub fn DividerPage() -> Element {
    rsx! {
        DocPage {
            title: "Divider",
            lead: rsx! {
                Text { "A horizontal or vertical rule, with an optional centered/positioned label." }
            },
            DocSection {
                title: "Horizontal",
                Divider {}
            }
            DocSection {
                title: "With a label",
                Divider { "OR" }
                Divider { label_position: "start", "Start" }
                Divider { label_position: "end", "End" }
            }
            DocSection {
                title: "Vertical",
                Flex {
                    direction: "row",
                    gap: "md",
                    sx: sx().height("48px"),
                    Text { "Left" }
                    Divider { orientation: "vertical" }
                    Text { "Right" }
                }
            }
        }
    }
}
