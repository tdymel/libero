use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Container, Text},
    sx::sx,
};

#[component]
pub fn ContainerPage() -> Element {
    rsx! {
        DocPage {
            title: "Container",
            lead: rsx! {
                Text {
                    "Centers content and caps its width at a breakpoint - wraps your main "
                    "content, not the whole page shell."
                }
            },
            DocSection {
                title: "Sizes",
                Container {
                    size: "sm",
                    sx: sx().background("grey.1").padding("16px"),
                    Text { "size: \"sm\" caps this container's width." }
                }
            }
        }
    }
}
