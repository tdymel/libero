//! `use_hotkeys`, as its docs page uses it: `mod+k` skipped in text entry, `ctrl+j` heard there.

use dioxus::prelude::*;
use libero::components::{Button, Menu, MenuEntry, MenuItem, use_menu};
use libero::hooks::{Hotkey, use_element, use_hotkeys};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-hotkeys", || rsx! { Hotkeys {} }),
    ("/use-hotkeys/within", || rsx! { Within {} }),
    ("/use-hotkeys/within-menu", || rsx! { WithinMenu {} }),
];

/// A bare `x` scoped to `#scope` and `#popup`, the latter standing in for a
/// portal; `#typed` shows what `#outside-field` received.
#[component]
fn Within() -> Element {
    let scope = use_element();
    let popup = use_element();
    let mut count = use_signal(|| 0);
    let mut typed = use_signal(String::new);
    use_hotkeys([Hotkey::new("x", move || count += 1)
        .include_editable(true)
        .within(scope)
        .within(popup)]);

    rsx! {
        button { id: "outside", "Outside" }
        input { id: "outside-field", oninput: move |event| typed.set(event.value()) }
        div { id: "scope", onmounted: scope.mount(), ..scope.attributes(),
            button { id: "inside", "Inside" }
            input { id: "inside-field" }
        }
        div { id: "popup", onmounted: popup.mount(), ..popup.attributes(),
            button { id: "in-popup", "In popup" }
        }
        p { id: "count", "{count}" }
        p { id: "typed", "{typed}" }
    }
}

fn menu_items() -> Vec<MenuEntry> {
    vec![
        MenuItem::new("Copy").into(),
        MenuItem::new("Share")
            .submenu(vec![MenuItem::new("Email").into()])
            .into(),
    ]
}

/// `ctrl+j` scoped to `#scope` only: its `Menu` opens inside it, the other outside.
#[component]
fn WithinMenu() -> Element {
    let scope = use_element();
    let inner = use_menu();
    let outer = use_menu();
    let mut count = use_signal(|| 0);
    use_hotkeys([Hotkey::new("ctrl+j", move || count += 1).within(scope)]);

    rsx! {
        div { id: "scope", onmounted: scope.mount(), ..scope.attributes(),
            Menu { state: inner, items: menu_items(),
                Button { attributes: inner.a11y_attributes(), "Inside" }
            }
        }
        div { id: "outside",
            Menu { state: outer, items: menu_items(),
                Button { attributes: outer.a11y_attributes(), "Outside" }
            }
        }
        p { id: "count", "{count}" }
    }
}

#[component]
fn Listener(mut open: Signal<u32>, mut typing: Signal<u32>) -> Element {
    use_hotkeys([
        Hotkey::new("mod+k", move || open += 1),
        Hotkey::new("ctrl+j", move || typing += 1).include_editable(true),
    ]);
    rsx! {}
}

#[component]
fn Sibling(mut count: Signal<u32>) -> Element {
    use_hotkeys([Hotkey::new("mod+k", move || count += 1)]);
    rsx! {}
}

#[component]
fn Hotkeys() -> Element {
    let open = use_signal(|| 0);
    let typing = use_signal(|| 0);
    let other = use_signal(|| 0);
    let mut listening = use_signal(|| true);

    rsx! {
        Sibling { count: other }
        p { id: "other", "{other}" }
        button { id: "plain", "Plain" }
        input { id: "field" }
        button { id: "remove", onclick: move |_| listening.set(false), "Remove" }
        if listening() {
            Listener { open, typing }
        }
        p { id: "open", "{open}" }
        p { id: "typing", "{typing}" }
    }
}
