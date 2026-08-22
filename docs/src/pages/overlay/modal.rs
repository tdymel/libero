use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap, indent};
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
            DocSection {
                title: "Usage",
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
}
