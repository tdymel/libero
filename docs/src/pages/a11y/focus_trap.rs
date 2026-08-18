use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Button, Flex, FocusTrap, Text};

#[component]
pub fn FocusTrapPage() -> Element {
    rsx! {
        DocPage {
            title: "Focus Trap",
            lead: rsx! {
                Text {
                    "Confines Tab/Shift+Tab cycling to its children - the same mechanism "
                    "Modal uses internally to keep keyboard focus inside an open dialog."
                }
            },
            DocSection {
                title: "Example",
                Text { "Tab through these buttons - focus stays inside the trap and wraps around." }
                FocusTrap {
                    Flex {
                        direction: "row",
                        gap: "sm",
                        Button { variant: "outlined", "First" }
                        Button { variant: "outlined", "Second" }
                        Button { variant: "outlined", "Third" }
                    }
                }
            }
        }
    }
}
