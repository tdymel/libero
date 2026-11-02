//! `Anchor`, one link per underline mode.

use dioxus::prelude::*;
use libero::components::{Anchor, Box, Flex, Header};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/anchor", || rsx! { AnchorPage {} }),
    ("/anchor/narrow", || rsx! { NarrowPage {} }),
];

/// A long word in a shrink-to-fit parent, and the docs' Header demo nav at a 320 px phone's column.
#[component]
fn NarrowPage() -> Element {
    let link = || {
        sx().padding("sm")
            .min_height("44px")
            .display("inline-flex")
            .align_items("center")
    };
    rsx! {
        div { id: "column", style: "width: 200px",
            div { style: "width: fit-content",
                Anchor { id: "long-word", to: "https://example.com",
                    "Versandkostenberechnungsgrundlagenverordnungsentwurfsfassungsmuster"
                }
            }
        }
        div { style: "width: 288px",
            Header { position: "static",
                "Libero"
                Box {
                    component: "nav",
                    aria_label: "Demo links",
                    sx: sx().display("flex").gap("md").with("margin-inline-start", "auto"),
                    Anchor { id: "docs-link", to: "https://example.com", sx: link(), "Docs" }
                    Anchor { id: "header-link", to: "https://example.com", sx: link(), "Header" }
                }
            }
        }
    }
}

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
