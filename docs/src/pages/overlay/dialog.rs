use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Dialog, Text};

const CONTENT: &str = r#"Text { "notes.md has changes you have not saved." }"#;

/// Outside a modal the close button calls `onclose`, so the snippet owns the
/// open state and a way back.
fn wrap_open(values: &DemoValues, code: &str) -> String {
    if values.str("close_button") != "true" {
        return code.to_string();
    }
    format!(
        "let mut open = use_signal(|| true);\n\n\
         rsx! {{\n    \
             if open() {{\n{}    }} else {{\n        \
                 Button {{ variant: \"outlined\", onclick: move |_| open.set(true), \"Reopen\" }}\n    \
             }}\n\
         }}",
        indent(&indent(code)),
    )
}

/// The open state needs a scope of its own: `Demo` calls its `render` closure
/// from its own.
#[component]
fn DialogDemo(titled: bool, close_button: bool, size: String, radius: String) -> Element {
    let mut open = use_signal(|| true);

    // Without the button the snippet has no open state, so nor does this.
    if close_button && !open() {
        return rsx! {
            Button { variant: "outlined", onclick: move |_| open.set(true), "Reopen" }
        };
    }
    rsx! {
        Dialog {
            title: titled.then(|| "Unsaved changes".to_string()),
            aria_label: (!titled).then(|| "Unsaved changes".to_string()),
            onclose: close_button.then_some(EventHandler::new(move |_| open.set(false))),
            size,
            radius,
            Text { "notes.md has changes you have not saved." }
        }
    }
}

#[component]
pub fn DialogPage() -> Element {
    rsx! {
        DocPage {
            title: "Dialog",
            source: "libero/src/components/overlay/dialog.rs",
            markdown: "/md/dialog.md",
            properties: vec![
                props("Dialog", vec![
                    prop("aria_label", "String").doc("Accessible name for the dialog; overrides title as the name."),
                    prop("title", "String").doc("Heading, and the accessible name unless aria_label overrides it."),
                    prop("close_button", "bool")
                        .default("in a modal, or with onclose")
                        .doc("Header button. Inside a modal it closes the modal, outside one it calls onclose."),
                    prop("onclose", "EventHandler<()>")
                        .doc("Called by the close button outside a modal. Inside one the button closes the modal instead."),
                    prop("close_label", "String").default("common.close").doc("Accessible name for the close button, e.g. \"Close cart\". Unset, the localization's `common.close` - \"Close\" in English."),
                    prop("radius", "Size").default("md").doc("Corner radius, a step on the radius scale. Anything else goes through `sx`."),
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
                    ". Outside a modal, as here, the button calls "
                    Code { source: "onclose" }
                    ". "
                    Code { source: "size" }
                    " caps its width from the dialog scale (md is 510px)."
                }
            },
            Demo {
                component: "Dialog",
                children_text: "",
                children_code: CONTENT.to_string(),
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl"]).default("md"),
                    // Unnamed, a dialog is announced as just "dialog": off
                    // names it through `aria_label` instead.
                    Control::switch("title")
                        .default("true")
                        .code(|_, values| match values.str("title").as_str() {
                            "true" => vec![r#"title: "Unsaved changes""#.to_string()],
                            _ => vec![r#"aria_label: "Unsaved changes""#.to_string()],
                        }),
                    // On by default once `onclose` is set, so that is the line.
                    Control::switch("close_button")
                        .default("true")
                        .code(|_, values| match values.str("close_button").as_str() {
                            "true" => vec!["onclose: move |_| open.set(false)".to_string()],
                            _ => vec![],
                        }),
                ],
                render: move |values: DemoValues| rsx! {
                    DialogDemo {
                        titled: values.str("title") == "true",
                        close_button: values.str("close_button") == "true",
                        size: values.str("size"),
                        radius: values.str("radius"),
                    }
                },
                wrap: Wrap(wrap_open),
                // The size scale runs to 900px: side by side, every step
                // from md up is the pane's width.
                wide_preview: true,
            }
        }
    }
}
