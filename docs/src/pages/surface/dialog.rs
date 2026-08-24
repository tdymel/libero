use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
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
            properties: vec![
                props("Dialog", vec![
                    prop("aria_label", "String").doc("Accessible name for the dialog."),
                    prop("radius", "ThemeAwareValue").default("md").doc("Corner radius - the radius scale, or any CSS length."),
                    prop("size", "ThemeAwareValue")
                        .default("md")
                        .doc("Caps the dialog's width from the dialog scale (md is 510px)."),
                    prop("variables", "Variables")
                        .doc("Layered onto Dialog's own - e.g. Drawer's anchor/size vars."),
                    prop("children", "Element").doc("The dialog's content."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "The dialog surface itself - padding, radius, shadow, and the role/"
                    "aria-modal wiring. Pair it with Modal for the portaled, backdrop-"
                    "dimmed, focus-trapped overlay behavior. "
                    Code { source: "size" }
                    " caps its width from the dialog scale (md is 510px)."
                }
            },
            Demo {
                component: "Dialog",
                children_text: "",
                children_code: CONTENT.to_string(),
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl"]).default("md"),
                ],
                render: move |values: DemoValues| rsx! {
                    Dialog {
                        aria_label: "Dialog surface",
                        size: values.str("size"),
                        radius: values.str("radius"),
                        sx: sx().margin("0"),
                        Title { size: "lg", "Dialog surface" }
                        Text { "Dialog rendered inline, without Modal's portal and backdrop." }
                    }
                },
            }
        }
    }
}
