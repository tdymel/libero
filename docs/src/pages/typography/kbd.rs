use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Kbd, Text};

#[component]
pub fn KbdPage() -> Element {
    rsx! {
        DocPage {
            title: "Kbd",
            properties: vec![props("Kbd", vec![
                prop("size", "Size")
                    .default("sm")
                    .doc("Font size. Everything else about the look is `Theme::kbd` only."),
                prop("children", "Element").doc("The key label."),
            ])],
            lead: rsx! {
                Text {
                    "Save with "
                    Kbd { "Ctrl" }
                    " + "
                    Kbd { "S" }
                    ". Renders a real "
                    Code { source: "<kbd>" }
                    ", styled entirely from the theme ("
                    Code { source: "Theme::kbd" }
                    ") - size is the only prop."
                }
            },
            Demo {
                component: "Kbd",
                children_text: "Ctrl",
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                ],
                render: move |values: DemoValues| rsx! {
                    Kbd { size: values.str("size"), "Ctrl" }
                },
            }
        }
    }
}
