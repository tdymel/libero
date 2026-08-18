use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Kbd, Text};

#[component]
pub fn KbdPage() -> Element {
    rsx! {
        DocPage {
            title: "Kbd",
            lead: rsx! {
                Text {
                    "Save with "
                    Kbd { "Ctrl" }
                    " + "
                    Kbd { "S" }
                    ". Renders a real "
                    Code { "<kbd>" }
                    ", styled entirely from the theme ("
                    Code { "Theme::kbd" }
                    ") - size is the only prop."
                }
            },
            DocSection {
                title: "Sizes",
                Flex {
                    direction: "row",
                    gap: "lg",
                    align: "center",
                    Kbd { size: "xs", "Ctrl" }
                    Kbd { size: "sm", "Ctrl" }
                    Kbd { size: "md", "Ctrl" }
                    Kbd { size: "lg", "Ctrl" }
                    Kbd { size: "xl", "Ctrl" }
                }
            }
        }
    }
}
