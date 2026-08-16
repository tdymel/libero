use dioxus::prelude::*;
use libero::{
    components::{Code, Divider, Flex, Text, Title},
    sx::sx,
};

const QUICK_START_EXAMPLE: &str = r#"fn App() -> Element {
    rsx! {
        LiberoProvider {
            Text { "Hello, Libero!" }
        }
    }
}"#;

#[component]
pub fn GettingStarted() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Getting Started" }
                Text {
                    "Libero is a Dioxus component library focused on developer experience, UX, accessibility, and configurability."
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Installation" }
                Text { "Add Libero to your project with cargo:" }
                Code { block: true, source: "cargo add libero", language: "shell" }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Quick Start" }
                Text {
                    "Wrap your app in "
                    Code { "LiberoProvider" }
                    " once, at the root - it registers the theme and every style your components use."
                }
                Code { block: true, source: QUICK_START_EXAMPLE, language: "rust" }
            }

            Flex {
                direction: "column",
                gap: "16px",
                Divider {}
                Text {
                    sx: sx().color("grey.6"),
                    "More documentation is on the way."
                }
            }
        }
    }
}
