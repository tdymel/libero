//! `Copy` bare, with a variant and a name of its own, with a description, and disabled.

use dioxus::prelude::*;
use libero::components::{Copy, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/copy", || rsx! { CopyPage {} }),
    ("/copy/timeout", || rsx! { CopyTimeout {} }),
];

/// Todo 3995: one that resets itself after a second and one that waits for the pointer or focus to leave.
#[component]
fn CopyTimeout() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            span { id: "plain", Copy { value: "plain" } }
            span { id: "timed", Copy { value: "timed", timeout: 1000 } }
        }
    }
}

#[component]
fn CopyPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            span { id: "bare", Copy { value: "bare value" } }
            // The visible text the contrast pass needs; an icon button has none.
            Text { "cargo add libero" }
            span { id: "named",
                Copy {
                    value: "cargo add libero",
                    aria_label: "Copy the install command",
                    variant: "outlined",
                }
            }
            span { id: "described",
                Copy { value: "cargo add libero", label: "Add libero to your project" }
            }
            span { id: "off", Copy { value: "off", variant: "filled", disabled: true } }
        }
    }
}
