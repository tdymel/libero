//! `Drawer`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Flex, Title},
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/drawer", || rsx! { DrawerPage {} }),
    ("/drawer/tall", || rsx! { TallDrawerPage {} }),
];

/// A drawer whose content outgrows the viewport.
#[component]
fn TallDrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            aria_label: Some("Menu".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                for line in 0..60 {
                    Anchor { to: "/", "Link {line}" }
                }
                Button { id: "drawer-close", variant: "text", onclick: move |_| s.close(), "Close" }
            }
        },
    );

    rsx! {
        Button {
            id: "open-drawer",
            onclick: move |_| {
                nav.open();
            },
            "Open menu"
        }
    }
}

#[component]
fn DrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            anchor: "end".into(),
            size: "sm".into(),
            aria_label: Some("Navigation".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Title { size: "lg", "Navigation" }
                Anchor { to: "/", "Home" }
                Button { id: "drawer-close", variant: "text", onclick: move |_| s.close(), "Close" }
            }
        },
    );

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open-drawer",
                variant: "outlined",
                onclick: move |_| {
                    nav.open();
                },
                "Open navigation"
            }
        }
    }
}
