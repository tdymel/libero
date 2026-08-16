use dioxus::prelude::*;
use libero::components::{Code, Flex, Mark, Text, Title};

#[component]
pub fn MarkPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Mark" }
                Text {
                    "Highlight "
                    Mark { "this chunk" }
                    " of the text. Renders a real "
                    Code { "<mark>" }
                    ", tinted with a light shade of the theme's "
                    Code { "warning" }
                    " color by default."
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Colors" }
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
