//! Fixtures for the overlay archetype beyond the `Modal` pilot: `Drawer`,
//! `Menu`, `Spotlight` and `Lightbox`.

use dioxus::prelude::*;
use libero::{
    components::{
        Anchor, Button, Flex, Menu, MenuEntry, MenuItem, SpotlightAction, SpotlightOptions, Text,
        Title, spotlight_filter, use_menu, use_spotlight,
    },
    hooks::{DrawerOptions, LightboxItem, LightboxOptions, ModalScope, use_drawer, use_lightbox},
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
        // Named, or every open logs the missing-name warning and the console
        // pass fails.
        // The theme's own name, so the fixture reads as a caller doing it
        // right rather than renaming the component.
        aria_label: Some("Command palette".into()),
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

static GALLERY: [Asset; 6] = [
    asset!("/assets/gallery/1.svg"),
    asset!("/assets/gallery/2.svg"),
    asset!("/assets/gallery/3.svg"),
    asset!("/assets/gallery/4.svg"),
    asset!("/assets/gallery/5.svg"),
    asset!("/assets/gallery/6.svg"),
];

/// The six pictures, each URL marked with `gallery`, so a second gallery of
/// the same files is fetched under URLs of its own and the test can tell
/// which gallery a request came from. The thumbnails get URLs of their own
/// too: the strip loads every one of them, and would hide what the stage's
/// lazy loading fetched.
fn gallery(name: &str) -> Vec<LightboxItem> {
    GALLERY
        .iter()
        .enumerate()
        .map(|(i, src)| {
            LightboxItem::new(format!("{src}?{name}"), format!("Picture {}", i + 1))
                .thumbnail(format!("{src}?{name}-thumbnail"))
                .caption(format!("Picture {} of the {name} gallery", i + 1))
        })
        .collect()
}

/// Two triggers into one viewer: the first picture, and the last - the far
/// end a gallery swap scrolls back from. `#swap-gallery` sits under the
/// modal, so only a script can press it: it is the second `open_with` while
/// the viewer is open (todo 323), which no control inside the viewer makes.
#[component]
pub fn LightboxPage() -> Element {
    let lightbox = use_lightbox(LightboxOptions {
        // Named, or every open logs the missing-name warning and the console
        // pass fails. The theme's own name, so the fixture reads as a caller
        // doing it right rather than renaming the component.
        aria_label: Some("Gallery".into()),
        ..LightboxOptions::default()
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "open-lightbox",
                variant: "outlined",
                onclick: move |_| {
                    lightbox.open_with(gallery("first"));
                },
                "Open the gallery"
            }
            Button {
                id: "open-lightbox-last",
                variant: "outlined",
                onclick: move |_| {
                    lightbox.open_with((gallery("first"), 5));
                },
                "Open the last picture"
            }
            Button {
                id: "swap-gallery",
                variant: "text",
                onclick: move |_| {
                    lightbox.open_with((gallery("second"), 2));
                },
                "Swap the gallery"
            }
        }
    }
}
