use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Mark, Text};

#[component]
pub fn MarkPage() -> Element {
    rsx! {
        DocPage {
            title: "Mark",
            lead: rsx! {
                Text {
                    "Highlight "
                    Mark { "this chunk" }
                    " of the text. Renders a real "
                    Code { "<mark>" }
                    ", tinted with a light shade of the theme's "
                    Code { "warning" }
                    " color by default."
                }
            },
            DocSection {
                title: "Colors",
                Text {
                    "Default: "
                    Mark { "warning" }
                    ", "
                    Mark { color: "primary", "primary" }
                    ", "
                    Mark { color: "success", "success" }
                    ", "
                    Mark { color: "error", "error" }
                    ", "
                    Mark { color: "info", "info" }
                    "."
                }
            }
        }
    }
}
