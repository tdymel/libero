//! `ProgressBar`, for the reduced-motion checks.

use dioxus::prelude::*;
use libero::components::{Button, Flex, ProgressBar, ProgressBarSegment, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/progress-bar", || rsx! { ProgressBarPage {} }),
    ("/progress-bar/segments", || rsx! { ProgressBarSegmentsPage {} }),
];

/// Todo 2167: 400px bars in segments from 0, 25 and 75, at 50, left to right and right to left.
#[component]
fn ProgressBarSegmentsPage() -> Element {
    let segments = vec![
        ProgressBarSegment::labeled(25.0, "Send"),
        ProgressBarSegment::labeled(75.0, "Verify"),
    ];
    rsx! {
        div { style: "width: 400px; display: flex; flex-direction: column; gap: 24px;",
            ProgressBar { id: "segments", aria_label: "Upload", value: 50.0, segments: segments.clone() }
            div { dir: "rtl",
                ProgressBar { id: "segments-rtl", aria_label: "Upload, right to left", value: 50.0, segments }
            }
        }
    }
}

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
