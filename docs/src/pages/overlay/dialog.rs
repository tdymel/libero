use dioxus::prelude::*;
use libero::{
    components::{Dialog, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn DialogPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Dialog" }
                Text {
                    "The dialog surface itself - padding, radius, shadow, and the role/"
                    "aria-modal wiring. Pair it with Modal for the portaled, backdrop-"
                    "dimmed, focus-trapped overlay behavior."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Example" }
                Dialog {
                    size: "sm",
                    sx: sx().margin("0"),
                    Title { variant: "h3", "Dialog surface" }
                    Text { "This is Dialog rendered inline, without Modal's portal/backdrop." }
                }
            }
        }
    }
}
