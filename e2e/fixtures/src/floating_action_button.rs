//! `FloatingActionButton` over a tall page: icon-only at `bottom-end`, a small one at
//! `top-start`, the same page under `dir="rtl"`, one above a fixed `NavigationBar`, one
//! that a switch hides, and one as a `Menu`'s trigger, also with its states behind switches.

use dioxus::prelude::*;
use libero::components::{
    FloatingActionButton, Menu, MenuItem, NavigationBar, NavigationBarItem, Pictogram, use_menu,
};
use libero::LiberoProvider;
use libero::hooks::{Align, Side};
use libero::theme::Direction;
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/floating-action-button", || rsx! { FabPage {} }),
    ("/floating-action-button/rtl", || rsx! { div { dir: "rtl", FabPage {} } }),
    ("/floating-action-button/bar", || rsx! { BarPage {} }),
    ("/floating-action-button/hide", || rsx! { HidePage {} }),
    ("/floating-action-button/menu", || rsx! { MenuPage {} }),
    ("/floating-action-button/menu-local-rtl", || rsx! { div { dir: "rtl", MenuPage {} } }),
    ("/floating-action-button/menu-states", || rsx! { MenuStatesPage {} }),
    ("/floating-action-button/menu-states-rtl", || rsx! {
        LiberoProvider { direction: Direction::Rtl, MenuStatesPage {} }
    }),
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

/// The button's states as a `Menu`'s trigger, each behind a switch.
#[component]
fn MenuStatesPage() -> Element {
    let menu = use_menu();
    let mut sent = use_signal(|| 0);
    let mut loading = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut hidden = use_signal(|| false);
    let mut tooltip = use_signal(|| false);
    let mut extended = use_signal(|| false);
    // A press that does nothing has no state to wait for: `tick` flushes, `opened` counts.
    let mut tick = use_signal(|| 0);
    let mut opened = use_signal(|| 0);
    let is_open = menu.is_open();
    use_effect(use_reactive!(|is_open| {
        if is_open {
            opened += 1;
        }
    }));
    rsx! {
        p { id: "sent", "{sent}" }
        p { id: "opened", "{opened}" }
        button { id: "tick", onclick: move |_| tick += 1, "{tick}" }
        button { id: "loading", onclick: move |_| loading.toggle(), "Loading" }
        button { id: "disabled", onclick: move |_| disabled.toggle(), "Disabled" }
        button { id: "hide", onclick: move |_| hidden.toggle(), "Hide" }
        button { id: "tooltip", onclick: move |_| tooltip.toggle(), "Tooltip" }
        button { id: "extended", onclick: move |_| extended.toggle(), "Extended" }
        div { style: "height: 3000px", "Page" }
        Menu {
            state: menu,
            side: Side::Top,
            align: Align::End,
            items: vec![
                MenuItem::new("Message").onselect(move |_| sent += 1).into(),
                MenuItem::new("Event").into(),
            ],
            FloatingActionButton {
                aria_label: (!extended()).then(|| "Create".to_string()),
                tooltip: tooltip(),
                loading: loading(),
                disabled: disabled(),
                hidden: hidden(),
                icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
                attributes: menu.a11y_attributes(),
                children: extended().then(|| rsx! { "Create" }),
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
