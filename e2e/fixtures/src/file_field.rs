//! `FileField`, as a dropzone taking several files.

use dioxus::prelude::*;
use libero::components::{FileField, Files, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/file-field", || rsx! { FileFieldPage {} })];

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
