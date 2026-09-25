use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text, TextField},
    hooks::{UndoHistory, use_history},
};

/// The field and its undo and redo buttons, as `UndoableNote` renders them.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let note = use_history(|| UndoHistory::new(String::new()), 500);

rsx! {
    Flex { direction: "column", align: "flex-start", gap: "sm",
        TextField {
            label: "Note",
            value: note.value().to_string(),
            oninput: move |next| note.merge(next),
        }
        Flex { direction: "row", gap: "sm",
            Button {
                variant: "outlined",
                disabled: !note.can_undo(),
                focusable_when_disabled: true,
                onclick: move |_| { note.undo(); },
                "Undo"
            }
            Button {
                variant: "outlined",
                disabled: !note.can_redo(),
                focusable_when_disabled: true,
                onclick: move |_| { note.redo(); },
                "Redo"
            }
            Button {
                variant: "text",
                onclick: move |_| note.reset(String::new()),
                "Clear"
            }
        }
    }
}"#
    .to_string()
}

#[component]
fn UndoableNote() -> Element {
    let note = use_history(|| UndoHistory::new(String::new()), 500);

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            TextField {
                label: "Note",
                value: note.value().to_string(),
                oninput: move |next| note.merge(next),
            }
            Flex { direction: "row", gap: "sm",
                Button {
                    variant: "outlined",
                    disabled: !note.can_undo(),
                    focusable_when_disabled: true,
                    onclick: move |_| {
                        note.undo();
                    },
                    "Undo"
                }
                Button {
                    variant: "outlined",
                    disabled: !note.can_redo(),
                    focusable_when_disabled: true,
                    onclick: move |_| {
                        note.redo();
                    },
                    "Redo"
                }
                Button {
                    variant: "text",
                    onclick: move |_| note.reset(String::new()),
                    "Clear"
                }
            }
        }
    }
}

#[component]
pub fn UseHistoryPage() -> Element {
    rsx! {
        DocPage {
            title: "History",
            source: "libero/src/hooks/history/mod.rs",
            markdown: "/md/use_history.md",
            accessibility: a11y()
                .handles([
                    "Nothing on screen: the hook keeps snapshots, your controls show them.",
                ])
                .must([
                    "Offer undo and redo from the keyboard as well, e.g. Ctrl+Z and Ctrl+Shift+Z on your own element; a button alone leaves a keyboard user tabbing away from the field.",
                    "Keep the Undo and Redo buttons focusable in the tab order, even at the end of the stack (`focusable_when_disabled`), and named by their text or `aria-label`.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_history(initial: impl FnOnce() -> UndoHistory<T>, group_ms: u64) -> HistoryHandle<T>" }
                    " keeps snapshots of a value to step back and forth through. "
                    Code { source: "push" }
                    " records a step of its own; "
                    Code { source: "merge" }
                    " folds rapid changes, such as typing, into one step. "
                    Code { source: "undo" }
                    ", "
                    Code { source: "redo" }
                    " and "
                    Code { source: "reset" }
                    " move through them, and "
                    Code { source: "value" }
                    ", "
                    Code { source: "can_undo" }
                    " and "
                    Code { source: "can_redo" }
                    " read them reactively."
                }
                Text {
                    "A group of merges closes on whichever comes first: "
                    Code { source: "group_ms" }
                    " without a merge, its size limit, or a "
                    Code { source: "seal" }
                    ", push, undo or redo. "
                    Code { source: "UndoHistory" }
                    " itself has no dioxus in it: build one with "
                    Code { source: "UndoHistory::new(value).with_cap(50).with_group_max(20)" }
                    " (defaults 200 steps and 50 changes per group) and use it on its own or hand it to the hook. "
                    "Each snapshot is held in an "
                    Code { source: "Rc" }
                    ", so cloning a history or a value is cheap."
                }
            },

            Demo {
                component: "use_history",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { UndoableNote {} },
                wrap: Wrap(code),
            }
        }
    }
}
