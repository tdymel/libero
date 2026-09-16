//! `Header`: a sticky banner with a nav, above a page that scrolls under it.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Header, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/header", || rsx! { HeaderPage {} }),
    ("/header-static", || rsx! { StaticPage {} }),
];

/// A sticky `lg` banner mounted before a static `xs` header, which must not
/// publish its height over the banner's.
#[component]
fn StaticPage() -> Element {
    rsx! {
        Header { id: "banner", size: "lg", "Banner" }
        main {
            Header { id: "nested", size: "xs", position: "static", "Nested" }
        }
    }
}

#[component]
fn HeaderPage() -> Element {
    rsx! {
        Header { id: "banner",
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
