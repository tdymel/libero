//! `use_hotkeys`, as its docs page uses it: `mod+k` skipped in text entry, `ctrl+j` heard there.

use dioxus::prelude::*;
use libero::components::{Button, HoverCard, Menu, MenuEntry, MenuItem, use_menu};
use libero::hooks::{Hotkey, PopoverOptions, Side, use_element, use_hotkeys, use_popover};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-hotkeys", || rsx! { Hotkeys {} }),
    ("/use-hotkeys/within", || rsx! { Within {} }),
    ("/use-hotkeys/within-menu", || rsx! { WithinMenu {} }),
    (
        "/use-hotkeys/within-hover-card",
        || rsx! { WithinHoverCard {} },
    ),
    ("/use-hotkeys/within-popover", || rsx! { WithinPopover {} }),
];

/// An app's own open `use_popover`, its box holding `#{id}-box`.
#[component]
fn OwnPopover(id: String) -> Element {
    let anchor = use_element();
    let popover = use_popover(anchor, true, PopoverOptions::new(4.0, 8.0));
    popover.show(Some(rsx! {
        div {
            onmounted: popover.floating().mount(),
            style: popover.style(),
            ..popover.floating_events(),
            button { id: "{id}-box", "In box" }
        }
    }));
    rsx! {
        button { onmounted: anchor.mount(), ..popover.anchor_events(), "{id}" }
    }
}

/// `ctrl+j` scoped to `#scope` only, a `use_popover` anchored inside it and one outside.
#[component]
fn WithinPopover() -> Element {
    let scope = use_element();
    let mut count = use_signal(|| 0);
    use_hotkeys([Hotkey::new("ctrl+j", move || count += 1).within(scope)]);

    rsx! {
        div { id: "scope", onmounted: scope.mount(), ..scope.attributes(),
            OwnPopover { id: "inside" }
        }
        div { style: "margin-top: 120px",
            OwnPopover { id: "outside" }
        }
        p { id: "count", "{count}" }
    }
}

/// `ctrl+j` scoped to `#scope` only, a forced-open `HoverCard` inside it and one outside.
#[component]
fn WithinHoverCard() -> Element {
    let scope = use_element();
    let mut count = use_signal(|| 0);
    use_hotkeys([Hotkey::new("ctrl+j", move || count += 1).within(scope)]);

    rsx! {
        div { id: "scope", onmounted: scope.mount(), ..scope.attributes(),
            HoverCard { aria_label: "Inside", open: true, side: Side::Bottom,
                content: rsx! { Button { id: "in-card", "In card" } },
                Button { "Inside" }
            }
        }
        div { id: "outside", style: "margin-top: 160px",
            HoverCard { aria_label: "Outside", open: true, side: Side::Bottom,
                content: rsx! { Button { id: "out-card", "Out card" } },
                Button { "Outside" }
            }
        }
        p { id: "count", "{count}" }
    }
}

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
