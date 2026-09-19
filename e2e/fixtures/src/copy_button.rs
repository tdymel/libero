//! `CopyButton` bare, with a variant and a name of its own, and disabled.

use dioxus::prelude::*;
use libero::components::{CopyButton, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/copy-button", || rsx! { CopyButtonPage {} })];

#[component]
fn CopyButtonPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            span { id: "bare", CopyButton { value: "bare value" } }
            // The visible text the contrast pass needs; an icon button has none.
            Text { "cargo add libero" }
            span { id: "named",
                CopyButton {
                    value: "cargo add libero",
                    aria_label: "Copy the install command",
                    variant: "outlined",
                }
            }
            span { id: "off", CopyButton { value: "off", variant: "filled", disabled: true } }
        }
    }
}
