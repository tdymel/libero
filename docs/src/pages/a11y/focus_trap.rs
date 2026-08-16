use dioxus::prelude::*;
use libero::components::{Button, Flex, FocusTrap, Text, Title};

#[component]
pub fn FocusTrapPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Focus Trap" }
                Text {
                    "Confines Tab/Shift+Tab cycling to its children - the same mechanism "
                    "Modal uses internally to keep keyboard focus inside an open dialog."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Example" }
                Text { "Tab through these buttons - focus stays inside the trap and wraps around." }
                FocusTrap {
                    Flex {
                        direction: "row",
                        gap: "8px",
                        Button { variant: "outlined", "First" }
                        Button { variant: "outlined", "Second" }
                        Button { variant: "outlined", "Third" }
                    }
                }
            }
        }
    }
}
