use dioxus::prelude::*;
use libero::components::{Code, Flex, Text, Title};

#[component]
pub fn CodePage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Code" }
                Text {
                    "Inline "
                    Code { "code" }
                    " by default, or a "
                    Code { "pre" }
                    "-wrapped block."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Block" }
                Code { block: true, "cargo add libero" }
            }
        }
    }
}
