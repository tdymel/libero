use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Overlay, Text, Title},
    sx::sx,
};

#[component]
pub fn OverlayPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Overlay" }
                Text {
                    "Dims/blurs whatever is behind it - Modal renders one behind its content. "
                    "Defaults to position: fixed, spanning the whole viewport; overridden to "
                    "position: absolute below to stay contained in this demo."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Example" }
                Box {
                    sx: sx().position("relative").height("160px").background("grey.2"),
                    Text { sx: sx().padding("16px"), "Content behind the overlay" }
                    Overlay { sx: sx().position("absolute") }
                }
            }
        }
    }
}
