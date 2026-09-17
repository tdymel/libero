use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, CodeBlock, Dialog, Flex, Text},
    hooks::{ModalScope, use_modal},
};

const CONFIRM: &str = r#"#[component]
fn DeleteFile() -> Element {
    let confirm = use_modal(|s: ModalScope<(), bool>| rsx! {
        Dialog { title: "Delete notes.md?", size: "sm",
            Text { "This cannot be undone." }
            Button { variant: "text", onclick: move |_| s.close(), "Cancel" }
            Button { variant: "filled", color: "error", onclick: move |_| s.resolve(true), "Delete" }
        }
    });
    let mut answer = use_signal(|| "nothing yet");

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                confirm.open().onresult(move |deleted| {
                    answer.set(if deleted == Some(true) { "deleted" } else { "kept" });
                });
            },
            "Delete file"
        }
        Text { "Last answer: {answer}" }
    }
}"#;

/// `CONFIRM`, rendered.
#[component]
fn DeleteFile() -> Element {
    let confirm = use_modal(|s: ModalScope<(), bool>| {
        rsx! {
            Dialog { title: "Delete notes.md?", size: "sm",
                Text { "This cannot be undone." }
                Button { variant: "text", onclick: move |_| s.close(), "Cancel" }
                Button {
                    variant: "filled",
                    color: "error",
                    onclick: move |_| s.resolve(true),
                    "Delete"
                }
            }
        }
    });
    let mut answer = use_signal(|| "nothing yet");

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                confirm
                    .open()
                    .onresult(move |deleted| {
                        answer.set(if deleted == Some(true) { "deleted" } else { "kept" });
                    });
            },
            "Delete file"
        }
        Text { "Last answer: {answer}" }
    }
}

#[component]
pub fn UseModalPage() -> Element {
    rsx! {
        DocPage {
            title: "use_modal",
            source: "libero/src/components/overlay/use_modal.rs",
            markdown: "/md/use_modal.md",
            lead: rsx! {
                Text {
                    Code { source: "use_modal(render) -> ModalHandle<S, R>" }
                    " registers a modal and returns the handle that opens it. The modal can "
                    "take arguments "
                    Code { source: "S" }
                    " and answer with a result "
                    Code { source: "R" }
                    ", which the caller reads with "
                    Code { source: "onresult" }
                    " or "
                    Code { source: ".await" }
                    ". "
                    Anchor { to: Route::ModalPage {}, "Modal" }
                    " has the full story."
                }
            },

            DocSection {
                title: "Usage",
                Flex { direction: "column", align: "flex-start", gap: "sm", DeleteFile {} }
                CodeBlock { source: CONFIRM, language: "rust" }
                Text {
                    "A dismissal answers "
                    Code { source: "None" }
                    ", so Escape or a backdrop click reads as \"kept\" here. Call the hook in "
                    "a component that outlives every trigger, because the modal is portaled "
                    "from there."
                }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "The modal traps focus while it is open, and focus goes back to the "
                    "button that opened it when it closes. Open it from the handler of what "
                    "the user acted on, so that is the element remembered. The "
                    Code { source: "Dialog" }
                    " inside supplies the role, and its "
                    Code { source: "title" }
                    " is the accessible name."
                }
            }
        }
    }
}
