use crate::components::{Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Dialog, Modal, Text, Title};

/// `Modal` has no opinion on content - the `Dialog` inside is what carries the
/// role and the aria semantics, so it is the example's own subtree.
const CONTENT: &str = r#"Dialog {
    aria_label: "Example modal",
    Title { size: "lg", "Example modal" }
    Text { "Closes on Escape or by clicking the backdrop." }
    Button { variant: "outlined", onclick: move |_| open.set(false), "Close" }
}"#;

/// A modal exists only while it is open, so the trigger and the state behind
/// it are the example as much as the component is.
fn wrap_trigger(_: &DemoValues, code: &str) -> String {
    format!(
        "Button {{ variant: \"outlined\", onclick: move |_| open.set(true), \"Open modal\" }}\nif open() {{\n{}}}",
        indent(code)
    )
}

#[component]
pub fn ModalPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        DocPage {
            title: "Modal",
            source: "libero/src/components/overlay/modal.rs",
            markdown: "/md/modal.md",
            properties: vec![
                props("Modal", vec![
                    prop("onclose", "EventHandler<()>")
                        .doc("Called on Escape or a backdrop click - the caller's state, not Modal, decides whether it actually closes."),
                    prop("children", "Element").doc("Content, mounted only while open. Pair with Dialog for the role and aria-modal semantics."),
                ]),
                props("Dialog", vec![
                    prop("aria_label", "String").doc("Accessible name for the dialog."),
                    prop("radius", "ThemeAwareValue").default("md").doc("Corner radius - the radius scale, or any CSS length."),
                    prop("size", "ThemeAwareValue").default("md").doc("Caps the dialog's width."),
                    prop("variables", "Variables")
                        .doc("Layered onto Dialog's own - e.g. Drawer's anchor/size vars."),
                    prop("children", "Element").doc("The dialog's content."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A focus-trapped, dimmed layer that locks scroll. It takes no props "
                    "beyond "
                    Code { source: "onclose" }
                    " and its children - it is mounted only while open, so the caller's own "
                    "state is the switch, and it has no opinion on content: pair it with "
                    Code { source: "Dialog" }
                    " for the role and aria-modal semantics."
                }
            },
            Demo {
                component: "Modal",
                children_text: "",
                children_code: CONTENT.to_string(),
                fixed: vec!["onclose: move |_| open.set(false)".to_string()],
                controls: vec![],
                render: move |_: DemoValues| rsx! {
                    Button {
                        variant: "outlined",
                        onclick: move |_| open.set(true),
                        "Open modal"
                    }
                    if open() {
                        Modal {
                            onclose: move |_| open.set(false),
                            Dialog {
                                aria_label: "Example modal",
                                Title { size: "lg", "Example modal" }
                                Text { "Closes on Escape or by clicking the backdrop." }
                                Button {
                                    variant: "outlined",
                                    onclick: move |_| open.set(false),
                                    "Close"
                                }
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_trigger),
            }
        }
    }
}
