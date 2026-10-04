use crate::components::{align_of, side_of};
use dioxus::prelude::*;
use libero::components::{Flex, MenuEntry, MenuItem, Menubar, MenubarMenu, Text};

/// The code block's example as a component, so its signal stays out of `Demo`'s hook slots.
#[component]
pub fn MenubarDemo(
    side: String,
    align: String,
    size: String,
    radius: String,
    loop_focus: bool,
) -> Element {
    // demo-code: preamble start
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());
    let item =
        move |name: &'static str| -> MenuEntry { MenuItem::new(name).onselect(pick(name)).into() };
    let mut wrap = use_signal(|| true);

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
                MenuItem::new("Save")
                    .shortcut("Control+S")
                    .onselect(pick("Save"))
                    .into(),
            ],
        ),
        MenubarMenu::new(
            "Edit",
            vec![
                item("Undo"),
                item("Redo"),
                MenuEntry::Separator,
                item("Cut"),
                item("Copy"),
                item("Paste"),
                MenuEntry::Separator,
                MenuItem::new("Word wrap")
                    .checkbox(wrap())
                    .onselect(move |_| wrap.toggle())
                    .into(),
            ],
        ),
        MenubarMenu::new("View", vec![item("Zoom in"), item("Zoom out")]).disabled(true),
        MenubarMenu::new("Help", vec![item("Documentation"), item("About")]),
    ];
    // demo-code: preamble end

    rsx! {
        Flex {
            direction: "column",
            gap: "md",
            Menubar {
                aria_label: "Main",
                menus,
                side: side_of(&side),
                align: align_of(&align),
                size,
                radius,
                loop_focus,
            }
            Text { size: "sm", role: "status", "Last chosen: {last()}" }
        }
    }
}
