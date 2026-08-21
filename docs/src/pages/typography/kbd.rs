use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Kbd, Text};

#[component]
pub fn KbdPage() -> Element {
    rsx! {
        DocPage {
            title: "Kbd",
            lead: rsx! {
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
            },
            DocSection {
                title: "Usage",
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
}
