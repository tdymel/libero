//! An active `NavLink` with `scroll_into_view`, below the fold of its sidebar,
//! and a page of every `NavLink`, `Anchor` and `Burger` state, plus a `leading`/`trailing` link.

use dioxus::prelude::*;
use libero::components::{Anchor, Badge, Burger, Flex, Icon, NavLink, Paper, Text};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/nav-link", || rsx! { NavLinkPage {} }),
    ("/nav-link/states", || rsx! { NavStatesPage {} }),
    ("/nav-link/blocked", || rsx! { BlockedPage {} }),
    ("/nav-link/sections", || rsx! { NavSectionsPage {} }),
];

/// A link with both sections, at the width a sidebar gives it.
#[component]
fn NavSectionsPage() -> Element {
    rsx! {
        Flex { direction: "column", max_width: "240px",
            NavLink {
                id: "sections",
                to: "/nav-link/sections",
                leading: rsx! { Icon { size: "sm", svg: lucide::bookmark::outlined } },
                trailing: rsx! { Badge { variant: "tonal", size: "sm", circle: true, "3" } },
                "Inbox"
            }
        }
    }
}

/// Links whose target would run script on click, and one ordinary link.
#[component]
fn BlockedPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Anchor { id: "javascript", to: "javascript:window.ranScript = true", "Script" }
            Anchor { id: "spaced", to: " Java\tScript:window.ranScript = true", "Spaced" }
            Anchor { id: "data", to: "data:text/html,<script>window.ranScript = true</script>", "Data" }
            NavLink { id: "vbscript", to: "VBScript:msgbox(1)", "Visual Basic" }
            Anchor { id: "ordinary", to: "https://example.com", "Ordinary" }
        }
    }
}

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
                NavLink { id: "not-current", to: "/nav-link/states", active: false, "Not current" }
                NavLink { id: "auto", to: "/nav-link/states", "Auto" }
                NavLink { id: "disabled", to: "/nav-link", disabled: true, "Disabled" }
                NavLink { id: "disabled-active", to: "/nav-link", active: true, disabled: true, "Disabled active" }
                NavLink {
                    id: "described-active",
                    to: "/nav-link/states",
                    active: true,
                    description: "Where you are",
                    "Described active"
                }
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
                Anchor { id: "external", to: "https://example.com", target: "_blank", new_tab_icon: true, "the external one" }
                "."
            }
            Text { id: "quiet-prose",
                Anchor {
                    id: "quiet",
                    to: "https://example.com",
                    target: "_blank",
                    new_tab_icon: true,
                    new_tab_hint: false,
                    "the quiet one"
                }
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
