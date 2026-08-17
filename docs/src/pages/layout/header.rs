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
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { size: "xxl", "Header" }
                Text { "The page's banner landmark - always renders header. This page's own header uses one." }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Colors" }
                Flex {
                    direction: "column",
                    gap: "8px",
                    Header { position: "static", color: "primary", Text { sx: sx().color("white"), "Primary" } }
                    Header { position: "static", color: "success", Text { sx: sx().color("white"), "Success" } }
                    Header { position: "static", "Neutral (default)" }
                }
            }
        }
    }
}
