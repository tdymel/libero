//! `Loader`, for the reduced-motion checks.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Loader, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/loader", || rsx! { LoaderPage {} })];

/// One loader per variant, mounted by "Upload": the spin is motion an
/// interaction starts. Each sits beside the text that says what it means.
#[component]
fn LoaderPage() -> Element {
    let mut busy = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "upload",
                variant: "outlined",
                onclick: move |_| busy.toggle(),
                if busy() { "Cancel" } else { "Upload" }
            }
            if busy() {
                for variant in ["oval", "bars", "dots"] {
                    Flex { key: "{variant}", direction: "row", gap: "sm", align: "center",
                        Loader { id: "{variant}", variant }
                        Text { "Uploading ({variant})" }
                    }
                }
            }
        }
    }
}
