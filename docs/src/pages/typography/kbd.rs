use dioxus::prelude::*;
use libero::components::{Code, Flex, Kbd, Text, Title};

#[component]
pub fn KbdPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Kbd" }
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
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Sizes" }
                Flex {
                    direction: "row",
                    gap: "16px",
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
