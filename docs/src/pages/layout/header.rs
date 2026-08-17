use dioxus::prelude::*;
use libero::{
    components::{Flex, Header, Text, Title},
    sx::sx,
};

#[component]
pub fn HeaderPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Header" }
                Text { "The page's banner landmark - always renders header. This page's own header uses one." }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Colors" }
                Flex {
                    direction: "column",
                    gap: "sm",
                    Header { position: "static", color: "primary", Text { sx: sx().color("white"), "Primary" } }
                    Header { position: "static", color: "success", Text { sx: sx().color("white"), "Success" } }
                    Header { position: "static", "Neutral (default)" }
                }
            }
        }
    }
}
