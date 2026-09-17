use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, CodeBlock, Dialog, Flex, Text},
    hooks::{ModalScope, use_modal, use_modal_close},
};

const DONE: &str = r#"/// Factored out of the render closure, so it cannot capture the `ModalScope`.
#[component]
fn DoneButton() -> Element {
    let close = use_modal_close();

    rsx! {
        Button { variant: "filled", onclick: move |_| close.call(()), "Done" }
    }
}

#[component]
fn SaveNotes() -> Element {
    let saved = use_modal(|_: ModalScope<()>| rsx! {
        Dialog { title: "Saved", size: "sm",
            Text { "notes.md is saved." }
            DoneButton {}
        }
    });

    rsx! {
        Button { variant: "outlined", onclick: move |_| { saved.open(); }, "Save" }
    }
}"#;

/// `DONE`, rendered.
#[component]
fn DoneButton() -> Element {
    let close = use_modal_close();

    rsx! {
        Button { variant: "filled", onclick: move |_| close.call(()), "Done" }
    }
}

#[component]
fn SaveNotes() -> Element {
    let saved = use_modal(|_: ModalScope<()>| {
        rsx! {
            Dialog { title: "Saved", size: "sm",
                Text { "notes.md is saved." }
                DoneButton {}
            }
        }
    });

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                saved.open();
            },
            "Save"
        }
    }
}

#[component]
pub fn UseModalClosePage() -> Element {
    rsx! {
        DocPage {
            title: "use_modal_close",
            source: "libero/src/components/overlay/use_modal.rs",
            markdown: "/md/use_modal_close.md",
            lead: rsx! {
                Text {
                    Code { source: "use_modal_close() -> Callback<()>" }
                    " closes the modal it is rendered in. It is for a component factored out "
                    "of the "
                    Anchor { to: Route::UseModalPage {}, "use_modal" }
                    " closure, which cannot capture the "
                    Code { source: "ModalScope" }
                    ". "
                    Anchor { to: Route::ModalPage {}, "Modal" }
                    " has the full story."
                }
            },

            DocSection {
                title: "Usage",
                Flex { align: "flex-start", SaveNotes {} }
                CodeBlock { source: DONE, language: "rust" }
                Text {
                    "Closing this way is a dismissal, the same as Escape. A component that may "
                    "render outside a modal asks first with "
                    Code { source: "try_use_context::<ModalContext>()" }
                    ", as the Modal page shows."
                }
            }
        }
    }
}
