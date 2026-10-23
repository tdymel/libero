//! `Drawer`, for the overlay archetype.

use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Flex, Title},
    context::Dismiss,
    hooks::{DrawerOptions, ModalScope, use_drawer},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/drawer", || rsx! { DrawerPage {} }),
    ("/drawer/tall", || rsx! { TallDrawerPage {} }),
    ("/drawer/wide", || rsx! { WideDrawerPage {} }),
    ("/drawer/bottom-tall", || rsx! { BottomTallDrawerPage {} }),
    ("/drawer/guarded", || rsx! { GuardedDrawerPage {} }),
    ("/drawer/long-title", || rsx! { LongTitleDrawerPage {} }),
    ("/drawer/titled", || rsx! { TitledDrawerPage {} }),
    ("/drawer/bottom-swipe", || rsx! { BottomSwipeDrawerPage {} }),
    ("/drawer/close-button", || rsx! { CloseButtonDrawerPage {} }),
];

/// 3979: a header close button, which closes even though `ondismiss` refuses every reason.
#[component]
fn CloseButtonDrawerPage() -> Element {
    let mut reasons = use_signal(Vec::<String>::new);
    let nav = use_drawer(
        DrawerOptions {
            title: Some("Filters".into()),
            close_button: true,
            ondismiss: Some(Callback::new(move |reason: Dismiss| {
                reasons.push(format!("{reason:?}"));
                false
            })),
            ..Default::default()
        },
        |_: ModalScope<()>| rsx! { div { id: "drawer-body", "Filters" } },
    );

    rsx! {
        Button { id: "open-drawer", onclick: move |_| { nav.open(); }, "Open filters" }
        div { id: "reasons", {reasons.read().join(",")} }
    }
}

/// Todo 3590: a swipe down from the bottom drawer's top padding closes it, and says so.
#[component]
fn BottomSwipeDrawerPage() -> Element {
    let mut reasons = use_signal(Vec::<String>::new);
    let nav = use_drawer(
        DrawerOptions {
            anchor: "bottom".into(),
            size: "sm".into(),
            title: Some("Filters".into()),
            handle: true,
            ondismiss: Some(Callback::new(move |reason: Dismiss| {
                reasons.push(format!("{reason:?}"));
                true
            })),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                div { id: "drawer-body", "Swipe the bar down to close." }
                Button { id: "drawer-close", onclick: move |_| s.close(), "Done" }
            }
        },
    );

    rsx! {
        Button { id: "open-drawer", onclick: move |_| { nav.open(); }, "Open filters" }
        div { id: "reasons", {reasons.read().join(",")} }
    }
}

/// Todo 3004: `DrawerOptions::title` is the panel's heading and its name.
#[component]
fn TitledDrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            title: Some("Filters".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Button { id: "drawer-close", onclick: move |_| s.close(), "Done" }
            }
        },
    );

    rsx! {
        Button { id: "open-drawer", onclick: move |_| { nav.open(); }, "Open filters" }
    }
}

/// Todo 2985: a long word in a `Title` in the docs' column of drawer content.
#[component]
fn LongTitleDrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            aria_label: Some("Menu".into()),
            ..Default::default()
        },
        |_: ModalScope<()>| {
            rsx! {
                Flex { direction: "column", gap: "md", sx: sx().padding("16px"),
                    Title { id: "drawer-title", size: "lg", "Donaudampfschifffahrtsgesellschaft" }
                }
            }
        },
    );

    rsx! {
        Button { id: "open-drawer", onclick: move |_| { nav.open(); }, "Open menu" }
    }
}

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

/// Todo 3002: a bottom drawer whose size outgrows the viewport, as at 200% zoom.
#[component]
fn BottomTallDrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            anchor: "bottom".into(),
            size: "200rem".into(),
            aria_label: Some("Menu".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                for line in 0..60 {
                    div { Anchor { to: "/", "Link {line}" } }
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
