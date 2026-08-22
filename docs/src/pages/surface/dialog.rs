use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, or_unset};
use dioxus::prelude::*;
use libero::{
    components::{Code, Dialog, Text, Title},
    sx::sx,
};

const CONTENT: &str = r#"Title { size: "lg", "Dialog surface" }
Text { "Dialog rendered inline, without Modal's portal and backdrop." }"#;

/// Inline, the surface keeps `Modal`'s centering margin - and it names itself,
/// since nothing else here does.
const FIXED: [&str; 2] = [r#"aria_label: "Dialog surface""#, r#"sx: sx().margin("0")"#];

#[component]
pub fn DialogPage() -> Element {
    rsx! {
        DocPage {
            title: "Dialog",
            lead: rsx! {
                Text {
                    "The dialog surface itself - padding, radius, shadow, and the role/"
                    "aria-modal wiring. Pair it with Modal for the portaled, backdrop-"
                    "dimmed, focus-trapped overlay behavior. "
                    Code { source: "size" }
                    " caps its width from the dialog scale (md is 510px)."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Dialog",
                    children_text: "",
                    children_code: CONTENT.to_string(),
                    fixed: FIXED.map(str::to_string).to_vec(),
                    controls: vec![
                        Control::slider("size", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                        Control::slider("radius", ["auto", "xs", "sm", "md", "lg", "xl"]),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Dialog {
                            aria_label: "Dialog surface",
                            size: or_unset(values.str("size")),
                            radius: or_unset(values.str("radius")),
                            sx: sx().margin("0"),
                            Title { size: "lg", "Dialog surface" }
                            Text { "Dialog rendered inline, without Modal's portal and backdrop." }
                        }
                    },
                }
            }
        }
    }
}
