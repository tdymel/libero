//! `FloatingActionButton` over a tall page: icon-only at `bottom-end`, a small one at
//! `top-start`, the same page under `dir="rtl"`, one above a fixed `NavigationBar`, one
//! that a switch hides, and one as a `Menu`'s trigger.

use dioxus::prelude::*;
use libero::components::{
    FloatingActionButton, Menu, MenuItem, NavigationBar, NavigationBarItem, Pictogram, use_menu,
};
use libero::hooks::{Align, Side};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/floating-action-button", || rsx! { FabPage {} }),
    ("/floating-action-button/rtl", || rsx! { div { dir: "rtl", FabPage {} } }),
    ("/floating-action-button/bar", || rsx! { BarPage {} }),
    ("/floating-action-button/hide", || rsx! { HidePage {} }),
    ("/floating-action-button/menu", || rsx! { MenuPage {} }),
];

#[component]
fn HidePage() -> Element {
    let mut hidden = use_signal(|| false);
    rsx! {
        button { id: "hide", onclick: move |_| hidden.toggle(), "Hide" }
        FloatingActionButton {
            id: "fab",
            aria_label: "Create",
            hidden: hidden(),
            icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
        }
        button { id: "after", "After" }
    }
}

/// Actions go through a `Menu`, with the button as its trigger.
#[component]
fn MenuPage() -> Element {
    let menu = use_menu();
    let mut sent = use_signal(|| 0);
    rsx! {
        p { id: "sent", "{sent}" }
        Menu {
            state: menu,
            side: Side::Top,
            align: Align::End,
            items: vec![
                MenuItem::new("Message").onselect(move |_| sent += 1).into(),
                MenuItem::new("Event").into(),
            ],
            FloatingActionButton {
                aria_label: "Create",
                icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
                attributes: menu.a11y_attributes(),
            }
        }
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
