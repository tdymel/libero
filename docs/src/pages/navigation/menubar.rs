use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Kbd, MenuEntry, MenuItem, Menubar, MenubarMenu, Text},
    hooks::{Align, Side},
};

/// Everything above the `rsx!`: what a pick does, and the menus themselves -
/// `generate_code` only emits props, and the list is a `let`.
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
            .toggled(wrap())
            .onselect(move |_| wrap.toggle())
            .into(),
    ]),
    MenubarMenu::new("View", vec![item("Zoom in"), item("Zoom out")]).disabled(true),
    MenubarMenu::new("Help", vec![item("Documentation"), item("About")]),
];

"#;

fn wrap(_: &DemoValues, source: &str) -> String {
    format!(
        "{PREAMBLE}rsx! {{\n    Flex {{\n        direction: \"column\",\n        gap: \"md\",\n{}        Text {{ size: \"sm\", \"Last chosen: {{last()}}\" }}\n    }}\n}}",
        indent(&indent(source))
    )
}

fn side_of(value: &str) -> Side {
    match value {
        "top" => Side::Top,
        _ => Side::Bottom,
    }
}

fn align_of(value: &str) -> Align {
    match value {
        "center" => Align::Center,
        "end" => Align::End,
        _ => Align::Start,
    }
}

/// `side` and `align` are the popover's enums, not strings, so the default
/// printer's `side: "top"` would not compile.
fn enum_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    if value == control.default {
        return vec![];
    }
    let (kind, variant) = match control.name {
        "side" => ("Side", format!("{:?}", side_of(&value))),
        _ => ("Align", format!("{:?}", align_of(&value))),
    };
    vec![format!("{}: {kind}::{variant}", control.name)]
}

/// The whole example from the code block, as a component: the signal lives
/// here rather than in `Demo`'s render closure, whose hook slots it would land
/// in.
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
                    .toggled(wrap())
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
            Text { size: "sm", "Last chosen: {last()}" }
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
                        .doc("The top-level menus, in order. Required."),
                    prop("aria_label", "String")
                        .doc("The bar's accessible name - `role=\"menubar\"` needs one. Required."),
                    prop("loop_focus", "bool")
                        .default("theme.menubar.loop_focus")
                        .doc("Whether the arrow keys wrap at the ends - along the bar, and down each menu."),
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
                ]),
                props("MenubarMenu", vec![
                    prop("new(label, items)", "String, Vec<MenuEntry>")
                        .doc("The trigger's text - also what typeahead on the bar matches - and exactly `Menu`'s items."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("The trigger stays in view and in the arrow order, and opens nothing."),
                ]).without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "A row of menus - the WAI-ARIA menubar, a desktop application's File, Edit "
                    "and View. Each menu is a "
                    Code { source: "Menu" }
                    " with the same items; the bar owns which one is open and which trigger "
                    "is its single tab stop, so you pass data and nothing else. Click a "
                    "trigger to open its menu; while one is open, the pointer on another "
                    "trigger switches to it."
                }
                Text {
                    "It belongs in an application - an editor, an IDE. For a page's own "
                    "navigation use links; for one set of actions, a single "
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
                    Control::toggle("side", ["bottom", "top"])
                        .default("bottom")
                        .code(enum_code),
                    Control::toggle("align", ["start", "center", "end"])
                        .default("start")
                        .code(enum_code),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
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
            DocSection {
                title: "Accessibility",
                Text {
                    "The bar is one tab stop. " Kbd { "←" } " " Kbd { "→" }
                    " move along it, a disabled trigger included (it takes focus and opens nothing), and " Kbd { "Home" } " "
                    Kbd { "End" } " go to the ends. " Kbd { "Enter" } " " Kbd { "Space" }
                    " and " Kbd { "↓" } " open a menu on its first item, " Kbd { "↑" }
                    " on its last. In an open menu, " Kbd { "←" } " and " Kbd { "→" }
                    " move to the neighbouring menu - unless a submenu answers them first. "
                    Kbd { "Esc" } " closes the menu and returns to its trigger; " Kbd { "Tab" }
                    " closes it and leaves the bar. Typing jumps to a trigger by its label. Everything else is "
                    Code { source: "Menu" }
                    "'s."
                }
            }
        }
    }
}
