//! `Menu`, for the overlay archetype.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Menu, MenuEntry, MenuItem, use_menu};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/menu", || rsx! { MenuPage {} }),
    ("/menu-open-on-mount", || rsx! { MenuOpenOnMountPage {} }),
    ("/menu-submenu-reopen", || rsx! { MenuSubmenuReopenPage {} }),
    ("/menu-choices", || rsx! { MenuChoicesPage {} }),
];

/// Radio items with one checked, a toggle with a shortcut, and a label longer
/// than a phone is wide.
#[component]
fn MenuChoicesPage() -> Element {
    let menu = use_menu();
    let mut sort = use_signal(|| "Name");
    let mut grid = use_signal(|| false);
    let choice = move |label: &'static str| {
        MenuItem::new(label)
            .checked(sort() == label)
            .onselect(move |_| sort.set(label))
            .into()
    };
    let items = vec![
        MenuEntry::Group {
            label: "Sort by".into(),
            items: vec![choice("Name"), choice("Date"), choice("Size")],
        },
        MenuEntry::Separator,
        MenuItem::new("Show grid")
            .toggled(grid())
            .shortcut("Control+G")
            .onselect(move |_| grid.toggle())
            .into(),
        MenuItem::new("Export every selected row as a comma separated values file")
            .onselect(|_| {})
            .into(),
    ];

    rsx! {
        p { id: "sort", "{sort}" }
        p { id: "grid", "{grid}" }
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "View" }
        }
    }
}

/// A group, a separator, a disabled item and a submenu - the shapes a menu's
/// accessibility tree can take.
#[component]
fn MenuPage() -> Element {
    let menu = use_menu();
    let items = vec![
        MenuEntry::Group {
            label: "Edit".into(),
            items: vec![
                MenuItem::new("Cut").onselect(|_| {}).into(),
                MenuItem::new("Paste")
                    .disabled(true)
                    .onselect(|_| {})
                    .into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Save").onselect(|_| {}).into(),
        MenuItem::new("Share")
            .submenu(vec![MenuItem::new("Email").onselect(|_| {}).into()])
            .into(),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

/// Todo 408: a menu opened before it mounts still has to move focus in.
#[component]
fn MenuOpenOnMountPage() -> Element {
    let menu = use_menu();
    use_hook(|| menu.open());
    let items = vec![
        MenuItem::new("Save").onselect(|_| {}).into(),
        MenuItem::new("Share").onselect(|_| {}).into(),
    ];

    rsx! {
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

/// A closed menu drops its submenu levels: a reopen must start them closed and
/// show the items changed meanwhile.
#[component]
fn MenuSubmenuReopenPage() -> Element {
    let menu = use_menu();
    let mut renamed = use_signal(|| false);
    let mut picked = use_signal(String::new);
    let label = if renamed() { "Post" } else { "Email" };
    let items = vec![
        MenuItem::new("Share")
            .submenu(vec![
                MenuItem::new(label)
                    .onselect(move |_| picked.set(label.to_string()))
                    .into(),
            ])
            .into(),
    ];

    rsx! {
        // Above the trigger, so the open levels do not cover it.
        Flex { direction: "column", gap: "md", max_width: "320px",
            button { id: "rename", onclick: move |_| renamed.set(true), "Rename" }
            p { id: "picked", "{picked}" }
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}
