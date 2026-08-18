use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Dialog, Text, Title},
    sx::sx,
};

#[component]
pub fn DialogPage() -> Element {
    rsx! {
        DocPage {
            title: "Dialog",
            lead: rsx! {
                Text {
                    "The dialog surface itself - padding, radius, shadow, and the role/"
                    "aria-modal wiring. Pair it with Modal for the portaled, backdrop-"
                    "dimmed, focus-trapped overlay behavior."
                }
            },
            DocSection {
                title: "Example",
                Dialog {
                    size: "sm",
                    sx: sx().margin("0"),
                    Title { size: "lg", "Dialog surface" }
                    Text { "This is Dialog rendered inline, without Modal's portal/backdrop." }
                }
            }
        }
    }
}
