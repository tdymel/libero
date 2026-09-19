//! An active `NavLink` with `scroll_into_view`, below the fold of its sidebar,
//! and a page of every `NavLink`, `Anchor` and `Burger` state.

use dioxus::prelude::*;
use libero::components::{Anchor, Burger, Flex, NavLink, Paper, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/nav-link", || rsx! { NavLinkPage {} }),
    ("/nav-link/states", || rsx! { NavStatesPage {} }),
];

#[component]
fn NavLinkPage() -> Element {
    rsx! {
        div { id: "sidebar", style: "height: 400px; overflow-y: auto;",
            div { style: "height: 1000px;" }
            NavLink {
                id: "here",
                to: "https://example.com",
                active: true,
                scroll_into_view: true,
                "Here"
            }
            div { style: "height: 1000px;" }
        }
    }
}

#[component]
fn NavStatesPage() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            nav { aria_label: "Sections",
                NavLink { id: "active", to: "/nav-link/states", active: true, "Active" }
                NavLink { id: "idle", to: "/nav-link", "Idle" }
                NavLink { id: "disabled", to: "/nav-link", disabled: true, "Disabled" }
                NavLink { id: "disabled-active", to: "/nav-link", active: true, disabled: true, "Disabled active" }
                NavLink {
                    id: "docs",
                    to: "/nav-link",
                    description: "Guides and API",
                    nested: rsx! {
                        NavLink { id: "install", to: "/nav-link", "Install" }
                        NavLink { id: "theming", to: "/nav-link", "Theming" }
                    },
                    "Docs"
                }
            }
            Text { id: "prose",
                "Read "
                Anchor { id: "inline", to: "/nav-link", "the inline link" }
                " in a sentence, or "
                Anchor { id: "external", to: "https://example.com", target: "_blank", "the external one" }
                "."
            }
            Flex { gap: "md", align: "center",
                for size in ["xs", "sm", "md", "lg", "xl", "xxl"] {
                    Burger { id: "burger-{size}", size, "aria-label": "Menu {size}" }
                }
            }
            Burger {
                id: "burger",
                open: open(),
                "aria-controls": "burger-panel",
                onclick: move |_| open.toggle(),
            }
            Paper { id: "burger-panel", hidden: (!open()).then_some(true), "Panel" }
        }
    }
}
