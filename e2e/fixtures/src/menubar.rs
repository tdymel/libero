//! `Menubar`, for the `RovingTabindex` archetype.

use dioxus::prelude::*;
use libero::components::{Flex, MenuEntry, MenuItem, Menubar, MenubarMenu};

use crate::Routes;

pub const ROUTES: Routes = &[("/menubar", || rsx! { MenubarPage {} })];

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
