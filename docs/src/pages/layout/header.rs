use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Flex, Header, Text},
    sx::sx,
};

#[component]
pub fn HeaderPage() -> Element {
    rsx! {
        DocPage {
            title: "Header",
            lead: rsx! {
                Text { "The page's banner landmark - always renders header. This page's own header uses one." }
            },
            DocSection {
                title: "Colors",
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
