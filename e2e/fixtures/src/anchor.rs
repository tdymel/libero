//! `Anchor`, one link per underline mode.

use dioxus::prelude::*;
use libero::components::{Anchor, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/anchor", || rsx! { AnchorPage {} })];

#[component]
fn AnchorPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", align: "flex-start",
            for underline in ["always", "hover", "not-hover", "never"] {
                Anchor { key: "{underline}", id: "{underline}", to: "https://example.com", underline, "{underline}" }
            }
        }
    }
}
