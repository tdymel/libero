//! Fixtures for the overlay archetype beyond the `Modal` pilot: `Drawer`,
//! `Menu` and `Spotlight`.

use dioxus::prelude::*;
use libero::{
    components::{
        Anchor, Button, Flex, Menu, MenuEntry, MenuItem, SpotlightAction, SpotlightOptions, Text,
        Title, spotlight_filter, use_menu, use_spotlight,
    },
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

#[component]
pub fn DrawerPage() -> Element {
    let nav = use_drawer(
        DrawerOptions {
            anchor: "right".into(),
            size: "sm".into(),
            aria_label: Some("Navigation".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| {
            rsx! {
                Title { size: "lg", "Navigation" }
                Anchor { to: "/", "Home" }
                Button { variant: "text", onclick: move |_| s.close(), "Close" }
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

/// A group, a separator, a disabled item and a submenu - the shapes a menu's
/// accessibility tree can take.
#[component]
pub fn MenuPage() -> Element {
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

/// The hook lives on the page, which outlives the trigger, as its docs ask.
#[component]
pub fn SpotlightPage() -> Element {
    let all = use_hook(|| {
        vec![
            SpotlightAction::new("Home")
                .group("Pages")
                .description("The start page"),
            SpotlightAction::new("Changelog").group("Pages"),
            SpotlightAction::new("New file")
                .group("Commands")
                .shortcut("Ctrl N"),
        ]
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        ..Default::default()
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open-spotlight",
                variant: "outlined",
                onclick: move |_| spotlight.open(),
                "Open the palette"
            }
            Text { size: "sm", "Or press Ctrl K." }
        }
    }
}
