use crate::components::{
    Control, Demo, DemoValues, DocPage, Wrap, a11y, align_of, indent, prop, props, side_of,
};
use dioxus::prelude::*;
use libero::components::{
    Code, Flex, MenuEntry, MenuItem, Menubar, MenubarMenu, MenubarPart, Text,
};

/// Everything above the `rsx!`, the pick handler and the menus. `generate_code`
/// only emits props, and the list is a `let`.
const PREAMBLE: &str = r#"let mut last = use_signal(|| String::from("nothing yet"));
let pick = move |name: &'static str| move |_| last.set(name.to_string());
let item = move |name: &'static str| -> MenuEntry {
    MenuItem::new(name).onselect(pick(name)).into()
};
let mut wrap = use_signal(|| true);

let menus = vec![
    MenubarMenu::new("File", vec![
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
    ]),
    MenubarMenu::new("Edit", vec![
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
    ]),
    MenubarMenu::new("View", vec![item("Zoom in"), item("Zoom out")]).disabled(true),
    MenubarMenu::new("Help", vec![item("Documentation"), item("About")]),
];

"#;

fn wrap(_: &DemoValues, source: &str) -> String {
    format!(
        "{PREAMBLE}rsx! {{\n    Flex {{\n        direction: \"column\",\n        gap: \"md\",\n{}        Text {{ size: \"sm\", role: \"status\", \"Last chosen: {{last()}}\" }}\n    }}\n}}",
        indent(&indent(source))
    )
}

/// The code block's example as a component, so its signal stays out of `Demo`'s hook slots.
#[component]
fn MenubarDemo(
    side: String,
    align: String,
    size: String,
    radius: String,
    loop_focus: bool,
) -> Element {
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

#[component]
pub fn MenubarPage() -> Element {
    rsx! {
        DocPage {
            title: "Menubar",
            source: "libero/src/components/navigation/menubar.rs",
            markdown: "/md/menubar.md",
            properties: vec![
                props("Menubar", vec![
                    prop("menus", "Vec<MenubarMenu>")
                        .default("required")
                        .doc("The top-level menus, in order."),
                    prop("aria_label", "String")
                        .default("required")
                        .doc("The bar's accessible name."),
                    prop("loop_focus", "bool")
                        .default("true")
                        .doc("Whether the arrow keys wrap at the ends, along the bar and down each menu."),
                    prop("side", "Side")
                        .default("Bottom")
                        .doc("Which side of its trigger every menu opens on. It flips when that side has no room."),
                    prop("align", "Align")
                        .default("Start")
                        .doc("Where each menu lines up along that side."),
                    prop("size", "Size")
                        .default("md")
                        .doc("The triggers' font and padding, and each menu's item size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("The triggers' and the menus' corner radius."),
                    prop("menu_parts", "Parts<MenuPart>")
                        .doc("Every menu's `parts`, the `Menu` page's Style API table. The menus open in a portal, out of the bar's `sx` and `parts`."),
                    prop("parts", "Parts<MenubarPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`. The menus take `menu_parts`."),
                ])
                .parts("MenubarPart", vec![
                    (MenubarPart::Trigger, "A menu's trigger button. `aria-expanded` is `true` while its menu is open."),
                ]),
                props("MenubarMenu", vec![
                    prop("new(label, items)", "String, Vec<MenuEntry>")
                        .doc("The trigger's text, which typeahead on the bar matches, and `Menu`'s items."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("The trigger stays in view and in the arrow order, and opens nothing."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters the bar, one tab stop. In an open menu: closes it and leaves the bar.")
                .key(["Left", "Right"], "On a trigger: moves along the bar, disabled triggers included, wrapping unless `loop_focus` is off. If a menu is open, the next one opens.")
                .key(["Home", "End"], "On a trigger: goes to the first or last trigger.")
                .key(["Enter", "Space", "Down"], "On a trigger: opens its menu on the first item.")
                .key(["Up"], "On a trigger: opens its menu on the last item.")
                .key(["Right"], "In an open menu: opens a submenu item's submenu, or else moves to the next menu.")
                .key(["Left"], "In an open menu: closes a submenu, or on the top level moves to the previous menu.")
                .key(["Escape"], "Closes the menu and returns focus to its trigger.")
                .handles([
                    "On a trigger, typing jumps to a trigger by its label.",
                    "A disabled trigger takes focus and opens nothing.",
                    "The rest works as in `Menu`, including `MenuItem`'s `shortcut` and `checkbox`.",
                ])
                .must(["Name the bar with `aria_label`. It is required."]),
            lead: rsx! {
                Text {
                    "A row of menus, like a desktop app's File, Edit and View. Each menu is a "
                    Code { source: "Menu" }
                    " with the same items. The bar keeps one menu open at most and is a single "
                    "tab stop. Click a trigger to open its menu. While one is open, hovering "
                    "another trigger switches to it."
                }
                Text {
                    "Use it in an app such as an editor. For page navigation, use links. For "
                    "one set of actions, use a single "
                    Code { source: "Menu" }
                    "."
                }
            },
            Demo {
                component: "Menubar",
                children_text: "",
                fixed: vec!["aria_label: \"Main\"".to_string(), "menus".to_string()],
                wrap: Wrap(wrap),
                controls: vec![
                    Control::side(["bottom", "top"]),
                    Control::align(),
                    Control::sizes("size").default("md"),
                    Control::sizes("radius").default("sm"),
                    Control::switch("loop_focus").default("true"),
                ],
                render: move |values: DemoValues| rsx! {
                    MenubarDemo {
                        side: values.str("side"),
                        align: values.str("align"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        loop_focus: values.str("loop_focus") == "true",
                    }
                },
            }
        }
    }
}
