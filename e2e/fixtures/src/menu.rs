//! `Menu`, for the overlay archetype.

use dioxus::prelude::*;
use libero::components::{
    Button, Flex, Menu, MenuEntry, MenuItem, MenuPart, Parts, Text, use_menu,
};
use libero::sx::sx;
use libero::theme::MENU_MAX_HEIGHT;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/menu", || rsx! { MenuPage {} }),
    ("/menu-open-on-mount", || rsx! { MenuOpenOnMountPage {} }),
    ("/menu-items-late", || rsx! { MenuItemsLatePage {} }),
    ("/menu-submenu-reopen", || rsx! { MenuSubmenuReopenPage {} }),
    ("/menu-choices", || rsx! { MenuChoicesPage {} }),
    ("/menu-row", || rsx! { MenuRowPage {} }),
    ("/menu-keep-open", || rsx! { MenuKeepOpenPage {} }),
    ("/menu-parts", || rsx! { MenuPartsPage {} }),
    ("/menu-tall/below", || rsx! { MenuTallPage { top: "100px" } }),
    (
        "/menu-tall/above",
        || rsx! { MenuTallPage { top: "calc(100vh - 100px)" } },
    ),
];

/// Forty items and no theme cap, the trigger at `top`: only the room on the
/// side it lands on holds the box (1739).
#[component]
fn MenuTallPage(top: &'static str) -> Element {
    let menu = use_menu();
    let items = (1..=40)
        .map(|n| MenuItem::new(format!("Item {n}")).onselect(|_| {}).into())
        .collect::<Vec<_>>();

    rsx! {
        div { position: "absolute", left: "16px", top,
            Menu {
                state: menu,
                items,
                sx: sx().var(MENU_MAX_HEIGHT, "100vh"),
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
        }
    }
}

/// `parts` styles the labels on every level, `sx` every level's box.
#[component]
fn MenuPartsPage() -> Element {
    let menu = use_menu();
    let items = vec![
        MenuItem::new("Share")
            .submenu(vec![MenuItem::new("Email").onselect(|_| {}).into()])
            .into(),
    ];

    rsx! {
        Menu {
            state: menu,
            items,
            parts: Parts::new().part(MenuPart::Label, sx().font_style("italic")),
            sx: sx().letter_spacing("2px"),
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
        }
    }
}

/// One item that keeps the menu open, one that closes it.
#[component]
fn MenuKeepOpenPage() -> Element {
    let menu = use_menu();
    let mut count = use_signal(|| 0);
    let items = vec![
        MenuItem::new("Zoom in")
            .keep_open()
            .onselect(move |_| count += 1)
            .into(),
        MenuItem::new("Save").onselect(|_| {}).into(),
    ];

    rsx! {
        p { id: "count", "{count}" }
        Menu {
            state: menu,
            items,
            Button { variant: "outlined", attributes: menu.a11y_attributes(), "View" }
        }
    }
}

/// Radio items with one checked, a toggle with a shortcut, a label longer than
/// a phone is wide, and a disabled item with a shortcut.
#[component]
fn MenuChoicesPage() -> Element {
    let menu = use_menu();
    let mut sort = use_signal(|| "Name");
    let mut grid = use_signal(|| false);
    let choice = move |label: &'static str| {
        MenuItem::new(label)
            .radio(sort() == label)
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
            .checkbox(grid())
            .shortcut("Control+G")
            .onselect(move |_| grid.toggle())
            .into(),
        MenuItem::new("Export every selected row as a comma separated values file")
            .onselect(|_| {})
            .into(),
        MenuItem::new("Print")
            .shortcut("Control+P")
            .disabled(true)
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

/// The docs demo's row: the trigger and the text beside it, centred.
#[component]
fn MenuRowPage() -> Element {
    let menu = use_menu();
    let items = vec![MenuItem::new("Save").onselect(|_| {}).into()];

    rsx! {
        Flex { direction: "row", align: "center", gap: "md",
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
            Text { id: "menu-row-text", size: "sm", "Last chosen: nothing yet" }
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

/// Todo 1789: opened empty, its items land a render later, as a load would.
#[component]
fn MenuItemsLatePage() -> Element {
    let menu = use_menu();
    let mut loaded = use_signal(|| false);
    use_effect(move || {
        if menu.is_open() && !*loaded.peek() {
            loaded.set(true);
        }
    });
    let items: Vec<MenuEntry> = match loaded() {
        true => vec![
            MenuItem::new("Save").onselect(|_| {}).into(),
            MenuItem::new("Share").onselect(|_| {}).into(),
        ],
        false => Vec::new(),
    };

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
