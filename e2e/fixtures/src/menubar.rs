//! `Menubar`, for the `RovingTabindex` archetype.

use dioxus::prelude::*;
use libero::components::{Flex, MenuEntry, MenuItem, Menubar, MenubarMenu};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/menubar", || rsx! { MenubarPage {} }),
    ("/menubar-docs", || rsx! { MenubarDocsPage {} }),
    ("/menubar-shrink", || rsx! { MenubarShrinkPage {} }),
];

/// `#drop`'s input event drops `View` and brings it back: no click, so an open
/// menu stays open through it.
#[component]
fn MenubarShrinkPage() -> Element {
    let mut view = use_signal(|| true);
    let item = |name: &str| -> MenuEntry { MenuItem::new(name).onselect(|_| {}).into() };
    let mut menus = vec![
        MenubarMenu::new("File", vec![item("New")]),
        MenubarMenu::new("Edit", vec![item("Undo")]),
    ];
    if view() {
        menus.push(MenubarMenu::new("View", vec![item("Zoom in")]));
    }

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Menubar { aria_label: "Editor", menus }
            input { id: "drop", oninput: move |_| view.toggle() }
        }
    }
}

/// The docs page's bar: a submenu, a disabled `View`, and `Help` after it.
#[component]
fn MenubarDocsPage() -> Element {
    let mut last = use_signal(|| String::from("nothing"));
    let item = move |name: &'static str| -> MenuEntry {
        MenuItem::new(name)
            .onselect(move |_| last.set(name.to_string()))
            .into()
    };
    let menus = vec![
        MenubarMenu::new(
            "File",
            vec![
                item("New"),
                item("Open"),
                MenuItem::new("Open recent")
                    .submenu(vec![item("notes.md"), item("todo.md")])
                    .into(),
                MenuEntry::Separator,
                item("Save"),
            ],
        ),
        MenubarMenu::new("Edit", vec![item("Undo"), item("Redo")]),
        MenubarMenu::new("View", vec![item("Zoom in")]).disabled(true),
        MenubarMenu::new("Help", vec![item("Documentation"), item("About")]),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            button { "before" }
            Menubar { aria_label: "Main", menus }
            p { id: "last", "{last}" }
            button { "after" }
        }
    }
}

/// Three menus, the middle one with a submenu row and a separator, so the
/// open-menu state has the shapes a real bar has.
#[component]
fn MenubarPage() -> Element {
    let item = |name: &str| -> MenuEntry { MenuItem::new(name).onselect(|_| {}).into() };
    let menus = vec![
        MenubarMenu::new(
            "File",
            vec![
                item("New"),
                item("Open"),
                MenuEntry::Separator,
                item("Save"),
            ],
        ),
        MenubarMenu::new(
            "Edit",
            vec![
                item("Undo"),
                item("Redo"),
                MenuItem::new("Find")
                    .submenu(vec![item("Find next")])
                    .into(),
            ],
        ),
        MenubarMenu::new("View", vec![item("Zoom in"), item("Zoom out")]),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            Menubar { aria_label: "Editor", menus }
        }
    }
}
