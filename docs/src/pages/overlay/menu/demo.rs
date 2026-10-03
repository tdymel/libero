use crate::components::{align_of, side_of};
use dioxus::prelude::*;
use libero::components::{Button, Flex, Menu, MenuEntry, MenuItem, Text, use_menu};

/// The whole example from the code block, as a component: the hooks live here
/// rather than in `Demo`'s render closure, whose hook slots they would land in.
#[component]
pub fn MenuDemo(
    side: String,
    align: String,
    size: String,
    radius: String,
    close_on_select: bool,
    loop_focus: bool,
    disabled: bool,
) -> Element {
    // demo-code: preamble start
    let menu = use_menu();
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());
    let mut sort = use_signal(|| "Name");
    let mut hidden = use_signal(|| false);
    let sort_by = move |name: &'static str| -> MenuEntry {
        MenuItem::new(name)
            .radio(sort() == name)
            .onselect(move |_| sort.set(name))
            .into()
    };

    let items = vec![
        MenuEntry::Group {
            label: "Edit".into(),
            items: vec![
                MenuItem::new("Cut")
                    .shortcut("Control+X")
                    .onselect(pick("Cut"))
                    .into(),
                MenuItem::new("Copy")
                    .shortcut("Control+C")
                    .onselect(pick("Copy"))
                    .into(),
                MenuItem::new("Paste")
                    .shortcut("Control+V")
                    .disabled(true)
                    .description("The clipboard is empty")
                    .onselect(pick("Paste"))
                    .into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Save").onselect(pick("Save")).into(),
        MenuItem::new("Save as").onselect(pick("Save as")).into(),
        MenuItem::new("Zoom in")
            .keep_open()
            .onselect(pick("Zoom in"))
            .into(),
        MenuItem::new("Share")
            .submenu(vec![
                MenuItem::new("Copy link")
                    .onselect(pick("Copy link"))
                    .into(),
                MenuItem::new("Email").onselect(pick("Email")).into(),
                MenuItem::new("Messages").onselect(pick("Messages")).into(),
            ])
            .into(),
        MenuEntry::Separator,
        MenuEntry::Group {
            label: "Sort by".into(),
            items: vec![sort_by("Name"), sort_by("Date"), sort_by("Size")],
        },
        MenuItem::new("Show hidden")
            .checkbox(hidden())
            .onselect(move |_| hidden.toggle())
            .into(),
        MenuItem::new("Documentation")
            .href("https://libero-ui.dev")
            .into(),
        MenuEntry::Separator,
        MenuItem::new("Delete").onselect(pick("Delete")).into(),
    ];
    // demo-code: preamble end

    rsx! {
        Flex {
            direction: "row",
            align: "center",
            gap: "md",
            Menu {
                state: menu,
                items,
                side: side_of(&side),
                align: align_of(&align),
                size,
                radius,
                close_on_select,
                loop_focus,
                disabled,
                Button {
                    variant: "outlined",
                    disabled: disabled.then_some(true),
                    attributes: menu.a11y_attributes(),
                    "Actions"
                }
            }
            Text { size: "sm", role: "status", "Last chosen: {last()}" }
        }
    }
}
