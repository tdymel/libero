//! `Drawer`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Flex, Title},
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/drawer", || rsx! { DrawerPage {} })];

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
