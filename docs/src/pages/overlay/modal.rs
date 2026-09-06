use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, CodeBlock, Dialog, Flex, Text},
    hooks::{ModalScope, Opening, use_modal},
};

const DIALOG_EXAMPLE: &str = r#"/// What the dialog can answer. A dismissal answers nothing, so the caller
/// matches on `Option<SaveChoice>` and "went back" is a case like any other.
#[derive(Clone, Copy, PartialEq)]
enum SaveChoice {
    Save,
    Discard,
}

/// `discard_label` is shared by every opening - the closure just captures it.
fn use_save_prompt(discard_label: &'static str) -> ModalHandle<String, SaveChoice> {
    use_modal(move |s: ModalScope<String, SaveChoice>| {
        let document = s.args();

        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: "sm",
                Text { "{document} has changes you have not saved." }
                Button { variant: "text", onclick: move |_| s.close(), "Keep editing" }
                Button {
                    variant: "filled",
                    color: "error",
                    onclick: move |_| s.resolve(SaveChoice::Discard),
                    "{discard_label}"
                }
                Button {
                    variant: "filled",
                    color: "primary",
                    onclick: move |_| s.resolve(SaveChoice::Save),
                    "Save"
                }
            }
        }
    })
}"#;

// snippet: after DIALOG_EXAMPLE
// snippet: item fn save() {}
// snippet: item fn discard() {}
const OPEN_EXAMPLE: &str = r#"let prompt = use_save_prompt("Discard");

Button {
    onclick: move |_| {
        prompt.open_with("notes.md").onresult(move |answer| match answer {
            Some(SaveChoice::Save) => save(),
            Some(SaveChoice::Discard) => discard(),
            None => {}   // dismissed - keep editing
        });
    },
    "Close editor"
}"#;

// snippet: after DIALOG_EXAMPLE
// snippet: item async fn save() {}
// snippet: item async fn discard() {}
// snippet: item async fn close_editor() {}
// snippet: let prompt = use_save_prompt("Discard");
// snippet: in Button { .., "Close editor" }
const AWAIT_EXAMPLE: &str = r#"onclick: move |_| async move {
    match prompt.open_with("notes.md").await {
        Some(SaveChoice::Save) => save().await,
        Some(SaveChoice::Discard) => discard().await,
        None => return,
    }
    close_editor().await;
}"#;

/// What the page's own dialog answers.
#[derive(Clone, Copy, PartialEq)]
enum SaveChoice {
    Save,
    Discard,
}

/// The page's own working dialog - the code beside it is what built this, so
/// the demo cannot drift from the example.
fn use_save_prompt(discard_label: &'static str) -> impl Fn(&str) -> Opening<SaveChoice> + Copy {
    let modal = use_modal(move |s: ModalScope<String, SaveChoice>| {
        let document = s.args();

        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: "sm",
                Flex {
                    direction: "column",
                    gap: "md",
                    Text { "{document} has changes you have not saved." }
                    Flex {
                        direction: "row",
                        gap: "sm",
                        Button { variant: "text", onclick: move |_| s.close(), "Keep editing" }
                        Button {
                            variant: "filled",
                            color: "error",
                            onclick: move |_| s.resolve(SaveChoice::Discard),
                            "{discard_label}"
                        }
                        Button {
                            variant: "filled",
                            color: "primary",
                            onclick: move |_| s.resolve(SaveChoice::Save),
                            "Save"
                        }
                    }
                }
            }
        }
    });

    move |document: &str| modal.open_with(document.to_string())
}

#[component]
pub fn ModalPage() -> Element {
    let prompt = use_save_prompt("Discard");
    let mut answer = use_signal(|| "-".to_string());

    rsx! {
        DocPage {
            title: "Modal",
            source: "libero/src/components/overlay/use_modal.rs",
            markdown: "/md/modal.md",
            lead: rsx! {
                Text {
                    "A modal is a hook, not a component. "
                    Code { source: "use_modal" }
                    " registers a render closure and returns a handle that opens it "
                    "wherever you need one - no open flag to thread, and the content is built "
                    "only while it is showing."
                }
            },
            DocSection {
                title: "Try it",
                Flex {
                    direction: "row",
                    align: "center",
                    gap: "md",
                    wrap: "wrap",
                    Button {
                        variant: "outlined",
                        onclick: move |_| {
                            prompt("notes.md").onresult(move |result| {
                                answer.set(
                                    match result {
                                        Some(SaveChoice::Save) => "saved",
                                        Some(SaveChoice::Discard) => "discarded",
                                        None => "dismissed - kept editing",
                                    }
                                    .to_string(),
                                );
                            });
                        },
                        "Close editor"
                    }
                    Text { "Last answer: " Code { source: answer() } }
                }
            }

            DocSection {
                title: "Building a dialog",
                Text {
                    "Wrap "
                    Code { source: "use_modal" }
                    " in a hook of your own: its parameters are shared by every opening, the "
                    Code { source: "ModalScope" }
                    " carries the arguments of the one being shown, and "
                    Code { source: "close()" }
                    " / "
                    Code { source: "resolve(value)" }
                    " end it. Make the result the dialog's own enum, not a "
                    Code { source: "bool" }
                    " - the caller then matches over what it can say, with a dismissal as "
                    Code { source: "None" }
                    " beside it."
                }
                CodeBlock { source: DIALOG_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Opening it",
                Text {
                    Code { source: "open_with" }
                    " takes anything that converts into the argument type and returns an "
                    Code { source: "Opening" }
                    " - that one showing. Attach the consequence to it and a later opening "
                    "cannot fire it."
                }
                CodeBlock { source: OPEN_EXAMPLE, language: "rust" }
                Text {
                    "It is also a future. Await it when the answer gates work that is already "
                    "async, or when several dialogs have to run in sequence."
                }
                CodeBlock { source: AWAIT_EXAMPLE, language: "rust" }
                Text {
                    Code { source: "open()" }
                    " skips the arguments when they are "
                    Code { source: "Default" }
                    ", and "
                    Code { source: "handle.close()" }
                    " closes whatever is showing. The handle is "
                    Code { source: "Copy" }
                    ", so a trigger elsewhere in the tree takes it as a prop - or as context, "
                    "if your hook provides it."
                }
            }

            DocSection {
                title: "Closing and accessibility",
                Text {
                    "Content in the render closure captures the "
                    Code { source: "ModalScope" }
                    "; a "
                    Code { source: "Dialog" }
                    " also closes itself from its own header button. Only a component factored "
                    "out of the closure needs "
                    Code { source: "use_modal_close()" }
                    "."
                }
                Text {
                    "Escape and a backdrop click dismiss it, settling the "
                    Code { source: "Opening" }
                    " with "
                    Code { source: "None" }
                    ", so a handler written for an answer never runs on a dismissal. Name the "
                    Code { source: "Dialog" }
                    " with its "
                    Code { source: "title" }
                    ", or "
                    Code { source: "aria_label" }
                    "."
                }
            }
        }
    }
}
