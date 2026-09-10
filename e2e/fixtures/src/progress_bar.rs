//! `ProgressBar`, for the reduced-motion checks.

use dioxus::prelude::*;
use libero::components::{Button, Flex, ProgressBar, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/progress-bar", || rsx! { ProgressBarPage {} })];

/// A determinate bar "Add 10%" moves, whose fill eases, and an indeterminate
/// one, whose fill sweeps.
#[component]
fn ProgressBarPage() -> Element {
    let mut done = use_signal(|| 40.0);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Text { "Upload" }
            ProgressBar { id: "upload", aria_label: "Upload", value: done() }
            Button {
                id: "add",
                variant: "outlined",
                onclick: move |_| done.set((done() + 10.0).min(100.0)),
                "Add 10%"
            }
            Text { "Sync" }
            ProgressBar { id: "sync", aria_label: "Sync", value: None }
        }
    }
}
