//! `Menu`, for the overlay archetype.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Menu, MenuEntry, MenuItem, use_menu};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/menu", || rsx! { MenuPage {} }),
    ("/menu-open-on-mount", || rsx! { MenuOpenOnMountPage {} }),
];

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
