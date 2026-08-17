use dioxus::prelude::*;
use libero::components::{Code, Flex, Kbd, Text, Title};

#[component]
pub fn KbdPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Kbd" }
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
                gap: "sm",
                Title { size: "xl", "Sizes" }
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
