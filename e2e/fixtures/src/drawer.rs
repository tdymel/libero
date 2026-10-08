//! `Drawer`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Flex, Title},
    context::Dismiss,
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/drawer", || rsx! { DrawerPage {} }),
    ("/drawer/tall", || rsx! { TallDrawerPage {} }),
    ("/drawer/wide", || rsx! { WideDrawerPage {} }),
    ("/drawer/guarded", || rsx! { GuardedDrawerPage {} }),
];

/// `ondismiss` keeps a dirty draft open and logs each reason (2600).
#[component]
fn GuardedDrawerPage() -> Element {
    let mut dirty = use_signal(|| false);
    let mut reasons = use_signal(Vec::<String>::new);
    let nav = use_drawer(
        DrawerOptions {
            aria_label: Some("Draft".into()),
            ondismiss: Some(Callback::new(move |reason: Dismiss| {
                reasons.push(format!("{reason:?}"));
                !dirty()
            })),
            ..Default::default()
        },
        move |_: ModalScope<()>| {
            rsx! {
                Button { id: "dirty", onclick: move |_| dirty.toggle(), "Toggle draft" }
            }
        },
    );

    rsx! {
        Button {
            id: "open-drawer",
            onclick: move |_| {
                nav.open();
            },
            "Open draft"
        }
        div { id: "reasons", {reasons.read().join(",")} }
    }
}

/// The widest size, wider than a phone.
#[component]
fn WideDrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            size: "xxl".into(),
            aria_label: Some("Menu".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
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
