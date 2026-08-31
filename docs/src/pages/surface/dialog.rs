use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Dialog, Text, Title},
    sx::sx,
};

const CONTENT: &str = r#"Title { size: "lg", "Dialog surface" }
Text { "Dialog rendered inline, without a modal's portal and backdrop." }"#;

/// Inline, the surface keeps the centering margin it wants in a modal - and it
/// names itself, since nothing else here does.
const FIXED: [&str; 2] = [r#"aria_label: "Dialog surface""#, r#"sx: sx().margin("0")"#];

#[component]
pub fn DialogPage() -> Element {
    rsx! {
        DocPage {
            title: "Dialog",
            source: "libero/src/components/surface/dialog.rs",
            markdown: "/md/dialog.md",
            properties: vec![
                props("Dialog", vec![
                    prop("aria_label", "String").doc("Accessible name for the dialog; overrides title as the name."),
                    prop("title", "String").doc("Heading, and the accessible name unless aria_label overrides it."),
                    prop("close_button", "bool")
                        .default("in a modal")
                        .doc("Header button that closes the surrounding modal. On by default inside one, where it has something to close."),
                    prop("close_label", "String").default("Close").doc("Accessible name for the close button."),
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
                    "aria-modal wiring. It is a "
                    Code { source: "Paper" }
                    ": the background, its focus contrast and the default radius are the "
                    "surface's, and only the chrome above is its own. Inside a modal it also names itself from "
                    Code { source: "title" }
                    " and closes itself from its own header button; open one with "
                    Code { source: "use_modal" }
                    ". "
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
                        Text { "Dialog rendered inline, without a modal's portal and backdrop." }
                    }
                },
            }
        }
    }
}
