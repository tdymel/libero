//! `FileField`, as a dropzone taking several files.

use dioxus::prelude::*;
use libero::components::{FileField, Files, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/file-field", || rsx! { FileFieldPage {} }),
    ("/file-field/input", || rsx! { FileInputPage {} }),
    ("/file-field/states", || rsx! { FileStatesPage {} }),
];

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
