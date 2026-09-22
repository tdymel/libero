//! `FileField`, as a dropzone taking several files.

use dioxus::prelude::*;
use libero::components::{Button, FileField, Files, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/file-field", || rsx! { FileFieldPage {} }),
    ("/file-field/input", || rsx! { FileInputPage {} }),
    ("/file-field/states", || rsx! { FileStatesPage {} }),
    ("/file-field/modes", || rsx! { FileModesPage {} }),
    ("/file-field/pick", || rsx! { FilePickPage {} }),
];

/// One file from the chooser, whose text shows once read: the bytes arrived.
#[component]
fn FilePickPage() -> Element {
    let mut files = use_signal(Files::default);
    let mut read = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "receipt",
                label: "Receipt",
                placeholder: "Pick a file",
                value: files(),
                onchange: move |next: Files| {
                    if let Some(file) = next.iter().next().cloned() {
                        spawn(async move {
                            read.set(file.read_string().await.unwrap_or_default());
                        });
                    }
                    files.set(next);
                },
            }
            p { id: "read", "{read}" }
        }
    }
}

/// One `multiple` field the buttons switch to read-only or disabled once the
/// unit has dropped its files in.
#[component]
fn FileModesPage() -> Element {
    let mut files = use_signal(Files::default);
    let mut mode = use_signal(|| "editable");

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "modes",
                label: "Papers",
                placeholder: "Pick files",
                multiple: true,
                readonly: mode() == "readonly",
                disabled: mode() == "disabled",
                value: files(),
                onchange: move |next: Files| files.set(next),
            }
            Button { id: "to-readonly", onclick: move |_| mode.set("readonly"), "Read-only" }
            Button { id: "to-disabled", onclick: move |_| mode.set("disabled"), "Disabled" }
        }
    }
}

/// The `Input` variant bare (no placeholder, todo 520), and required with an
/// error.
#[component]
fn FileStatesPage() -> Element {
    let mut bare = use_signal(Files::default);
    let mut contract = use_signal(Files::default);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "bare",
                label: "Receipt",
                value: bare(),
                onchange: move |next: Files| bare.set(next),
            }
            FileField {
                id: "contract",
                label: "Contract",
                placeholder: "Pick a file",
                required: true,
                status: "We cannot read that file.",
                multiple: true,
                value: contract(),
                onchange: move |next: Files| contract.set(next),
            }
        }
    }
}

/// The `Input` variant: dropped files become chips the arrows walk.
#[component]
fn FileInputPage() -> Element {
    let mut files = use_signal(Files::default);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "attachments",
                label: "Attachments",
                placeholder: "Pick files",
                multiple: true,
                value: files(),
                onchange: move |next: Files| files.set(next),
            }
        }
    }
}

/// Starts empty: a `FileData` only comes from a pick or a drop, so the unit
/// drops its files in.
#[component]
fn FileFieldPage() -> Element {
    let mut files = use_signal(Files::default);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "attachments",
                label: "Attachments",
                variant: "dropzone",
                multiple: true,
                value: files(),
                onchange: move |next: Files| files.set(next),
            }
        }
    }
}
