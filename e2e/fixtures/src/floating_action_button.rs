//! `FloatingActionButton` over a tall page: icon-only at `bottom-end`, a small one at
//! `top-start`, the same page under `dir="rtl"`, one above a fixed `NavigationBar`, and
//! a speed dial that a switch hides.

use dioxus::prelude::*;
use libero::components::{
    FloatingActionButton, FloatingActionButtonAction, NavigationBar, NavigationBarItem, Pictogram,
};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/floating-action-button", || rsx! { FabPage {} }),
    ("/floating-action-button/rtl", || rsx! { div { dir: "rtl", FabPage {} } }),
    ("/floating-action-button/bar", || rsx! { BarPage {} }),
    ("/floating-action-button/dial", || rsx! { DialPage {} }),
    ("/floating-action-button/dial/rtl", || rsx! { div { dir: "rtl", DialPage {} } }),
];

/// A speed dial, a hide switch for it, and a count of the action presses.
#[component]
fn DialPage() -> Element {
    let mut hidden = use_signal(|| false);
    let mut sent = use_signal(|| 0);
    rsx! {
        button { id: "hide", onclick: move |_| hidden.toggle(), "Hide" }
        p { id: "sent", "{sent}" }
        FloatingActionButton {
            id: "fab",
            aria_label: "Create",
            hidden: hidden(),
            icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
            actions: rsx! {
                FloatingActionButtonAction {
                    id: "message",
                    icon: rsx! { Pictogram { icon: lucide::mail::outlined } },
                    onclick: move |_| sent += 1,
                    label: "Message",
                }
                FloatingActionButtonAction {
                    id: "event",
                    icon: rsx! { Pictogram { icon: lucide::calendar::outlined } },
                    label: "Event",
                }
            },
        }
        button { id: "after", "After" }
    }
}

#[component]
fn FabPage() -> Element {
    rsx! {
        a { id: "before", href: "#before", "Before" }
        div { style: "height: 3000px", "Page" }
        FloatingActionButton {
            id: "fab",
            aria_label: "New message",
            icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
            onclick: |_| {},
        }
        FloatingActionButton {
            id: "small",
            aria_label: "Search",
            size: "xs",
            placement: "top-start",
            icon: rsx! { Pictogram { icon: lucide::search::outlined } },
        }
    }
}

#[component]
fn BarPage() -> Element {
    rsx! {
        div { style: "height: 2000px; padding-bottom: var(--lsx-navigation-bar-height)", "Page" }
        FloatingActionButton {
            id: "fab",
            icon: rsx! { Pictogram { icon: lucide::pencil::outlined } },
            "Compose"
        }
        NavigationBar { id: "bar", "aria-label": "Main", position: "fixed",
            NavigationBarItem { selected: true, onclick: |_| {}, icon: rsx! { Pictogram { icon: lucide::house::outlined } }, "Home" }
            NavigationBarItem { onclick: |_| {}, icon: rsx! { Pictogram { icon: lucide::search::outlined } }, "Search" }
        }
    }
}
