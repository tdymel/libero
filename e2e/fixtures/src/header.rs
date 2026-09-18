//! `Header`: a sticky banner with a nav, above a page that scrolls under it.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Header, Paper, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/header", || rsx! { HeaderPage {} }),
    ("/header-static", || rsx! { StaticPage {} }),
    ("/header-glass", || rsx! { GlassPage {} }),
];

/// A glass banner given a `color` it must drop, and a glass `Paper`.
#[component]
fn GlassPage() -> Element {
    rsx! {
        Header { id: "banner", color: "primary", glass: true, "Glass" }
        Paper { id: "card", glass: true, Text { "A glass card." } }
    }
}

/// A publishing sticky `lg` banner mounted before a static `xs` header and a
/// sticky `sm` one that did not opt in; neither may publish over the banner.
#[component]
fn StaticPage() -> Element {
    rsx! {
        Header { id: "banner", size: "lg", publish_height: true, "Banner" }
        main {
            Header { id: "nested", size: "xs", position: "static", "Nested" }
            Header { id: "demo", size: "sm", "Demo" }
        }
    }
}

#[component]
fn HeaderPage() -> Element {
    rsx! {
        Header { id: "banner", publish_height: true,
            nav { aria_label: "Main",
                Button { id: "home", variant: "standard", "Home" }
            }
        }
        Flex { direction: "column", gap: "xl", max_width: "320px",
            for index in 0..24 {
                Flex { key: "{index}", direction: "column", gap: "xl",
                    Button { id: "item-{index}", variant: "outlined", "Item {index}" }
                    Text { "Filler that makes the page scroll." }
                }
            }
        }
    }
}
