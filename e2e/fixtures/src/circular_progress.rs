//! `CircularProgress`: a determinate ring "Add 10%" moves and a spinning one.

use dioxus::prelude::*;
use libero::components::{Button, CircularProgress, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/circular-progress", || rsx! { CircularProgressPage {} })];

#[component]
fn CircularProgressPage() -> Element {
    let mut done = use_signal(|| 40.0);

    rsx! {
        Flex { direction: "column", gap: "md", align: "start",
            CircularProgress { id: "upload", aria_label: "Upload", value: done(), size: "xl", "{done}%" }
            Button {
                id: "add",
                variant: "outlined",
                onclick: move |_| done.set((done() + 10.0).min(100.0)),
                "Add 10%"
            }
            CircularProgress { id: "sync", aria_label: "Sync", value: None }
        }
    }
}
