use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Kbd, Menu, MenuEntry, MenuItem, Text, use_menu},
    hooks::{Align, Side},
};

/// Everything above the `rsx!`: the state, what a pick does, and the item
/// list itself - `generate_code` only emits props, and the list is a `let`.
const PREAMBLE: &str = r#"let menu = use_menu();
let mut last = use_signal(|| String::from("nothing yet"));
let pick = move |name: &'static str| move |_| last.set(name.to_string());
let mut sort = use_signal(|| "Name");
let sort_by = move |name: &'static str| -> MenuEntry {
    MenuItem::new(name)
        .checked(sort() == name)
        .onselect(move |_| sort.set(name))
        .into()
};

let items = vec![
    MenuEntry::Group {
        label: "Edit".into(),
        items: vec![
            MenuItem::new("Cut")
                .trailing(rsx! { Kbd { "Ctrl X" } })
                .onselect(pick("Cut"))
                .into(),
            MenuItem::new("Copy")
                .trailing(rsx! { Kbd { "Ctrl C" } })
                .onselect(pick("Copy"))
                .into(),
            MenuItem::new("Paste")
                .trailing(rsx! { Kbd { "Ctrl V" } })
                .disabled(true)
                .onselect(pick("Paste"))
                .into(),
        ],
    },
    MenuEntry::Separator,
    MenuItem::new("Save").onselect(pick("Save")).into(),
    MenuItem::new("Save as").onselect(pick("Save as")).into(),
    MenuItem::new("Share")
        .submenu(vec![
            MenuItem::new("Copy link").onselect(pick("Copy link")).into(),
            MenuItem::new("Email").onselect(pick("Email")).into(),
            MenuItem::new("Messages").onselect(pick("Messages")).into(),
        ])
        .into(),
    MenuEntry::Separator,
    MenuEntry::Group {
        label: "Sort by".into(),
        items: vec![sort_by("Name"), sort_by("Date"), sort_by("Size")],
    },
    MenuEntry::Separator,
    MenuItem::new("Delete").onselect(pick("Delete")).into(),
];

"#;

// snippet: after PREAMBLE
const TRIGGER: &str = r#"Button {
    variant: "outlined",
    attributes: menu.a11y_attributes(),
    "Actions"
}"#;

fn wrap(_: &DemoValues, source: &str) -> String {
    format!(
        "{PREAMBLE}rsx! {{\n    Flex {{\n        direction: \"row\",\n        gap: \"md\",\n{}        Text {{ size: \"sm\", \"Last chosen: {{last()}}\" }}\n    }}\n}}",
        indent(&indent(source))
    )
}

fn side_of(value: &str) -> Side {
    match value {
        "top" => Side::Top,
        "left" => Side::Left,
        "right" => Side::Right,
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

/// The whole example from the code block, as a component: the hooks live here
/// rather than in `Demo`'s render closure, whose hook slots they would land in.
#[component]
fn MenuDemo(
    side: String,
    align: String,
    size: String,
    radius: String,
    close_on_select: bool,
    loop_focus: bool,
    disabled: bool,
) -> Element {
    let menu = use_menu();
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());
    let mut sort = use_signal(|| "Name");
    let sort_by = move |name: &'static str| -> MenuEntry {
        MenuItem::new(name)
            .checked(sort() == name)
            .onselect(move |_| sort.set(name))
            .into()
    };

    let items = vec![
        MenuEntry::Group {
            label: "Edit".into(),
            items: vec![
                MenuItem::new("Cut")
                    .trailing(rsx! { Kbd { "Ctrl X" } })
                    .onselect(pick("Cut"))
                    .into(),
                MenuItem::new("Copy")
                    .trailing(rsx! { Kbd { "Ctrl C" } })
                    .onselect(pick("Copy"))
                    .into(),
                MenuItem::new("Paste")
                    .trailing(rsx! { Kbd { "Ctrl V" } })
                    .disabled(true)
                    .onselect(pick("Paste"))
                    .into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Save").onselect(pick("Save")).into(),
        MenuItem::new("Save as").onselect(pick("Save as")).into(),
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
        MenuEntry::Separator,
        MenuItem::new("Delete").onselect(pick("Delete")).into(),
    ];

    rsx! {
        Flex {
            direction: "row",
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
                    attributes: menu.a11y_attributes(),
                    "Actions"
                }
            }
            Text { size: "sm", "Last chosen: {last()}" }
        }
    }
}

#[component]
pub fn MenuPage() -> Element {
    rsx! {
        DocPage {
            title: "Menu",
            source: "libero/src/components/navigation/menu",
            markdown: "/md/menu.md",
            properties: vec![
                props("Menu", vec![
                    prop("state", "MenuState")
                        .doc("From `use_menu()`: the open state and the id the aria wiring is built from. Required."),
                    prop("items", "Vec<MenuEntry>")
                        .doc("The menu, in order: `MenuEntry::Item`, `MenuEntry::Group { label, items }` for a named section, and `MenuEntry::Separator`. Required."),
                    prop("children", "Element")
                        .doc("The trigger, carrying `menu.a11y_attributes()`. Its clicks and keys are caught on the wrapper they bubble to, so it needs no handler of its own."),
                    prop("side", "Side")
                        .default("Bottom")
                        .doc("Which side of the trigger the menu opens on. It flips when that side has no room."),
                    prop("align", "Align")
                        .default("Start")
                        .doc("Where the menu lines up along that side."),
                    prop("close_on_select", "bool")
                        .default("theme.menu.close_on_select")
                        .doc("Whether choosing an item closes the menu."),
                    prop("loop_focus", "bool")
                        .default("theme.menu.loop_focus")
                        .doc("Whether the arrow keys wrap from the last item to the first."),
                    prop("size", "Size")
                        .default("md")
                        .doc("Item height and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("The menu's corner radius. Items nest inside it with a radius tightened by the menu's padding."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("The trigger opens nothing. Disable the trigger too, which draws its own dimmed state."),
                    prop("onedge", "Option<Callback<MenuEdge>>")
                        .default("None")
                        .doc("Hears ← and → when no submenu answers them: ← on the top level, → on any item without a submenu. `Menubar` moves to the neighbouring menu with it."),
                ]),
                props("MenuItem", vec![
                    prop("new(label)", "String")
                        .doc("The visible text, the accessible name, and what typeahead matches."),
                    prop("onselect", "FnMut(())")
                        .doc("Runs when the item is chosen: a click, or Enter or Space on it."),
                    prop("submenu", "Vec<MenuEntry>")
                        .doc("Opens a second menu beside the item instead. An item either runs a command or opens a submenu - the later call replaces the earlier."),
                    prop("leading", "Element")
                        .doc("Before the label - an icon. It sits inside the item's button, so nothing interactive."),
                    prop("trailing", "Element")
                        .doc("At the far end - a shortcut hint. Nothing interactive, for the same reason."),
                    prop("checked", "bool")
                        .doc("Unset, a plain command. Set, it makes the item one choice of several: a `menuitemradio` announcing `aria-checked`, with a check before the label. Put the choices in one `Group`; keeping exactly one checked is yours. A menu holding a checked item opens on it."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Stays in the arrow-key order, cannot be chosen, and typeahead skips it."),
                ]).without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "A list of commands that drops from a trigger - the WAI-ARIA menu button. "
                    "The items are data, not children, so the menu owns their order: the arrow "
                    "keys and typeahead index a "
                    Code { source: "Vec" }
                    " rather than asking the page. "
                    Code { source: "use_menu()" }
                    " keeps the open state in your scope, and the trigger is your own "
                    Code { source: "Button" }
                    ", wired by "
                    Code { source: "menu.a11y_attributes()" }
                    ". The menu is a surface from the theme's "
                    Code { source: "paper" }
                    " defaults, portaled so no "
                    Code { source: "overflow: hidden" }
                    " ancestor clips it."
                }
                Text {
                    "Focus moves onto the items - one of them is tabbable at a time - so a "
                    "screen reader follows it. A submenu opens beside its item on "
                    Kbd { "→" }
                    ", a click, or the pointer resting on the item for "
                    Code { source: "theme.menu.submenu_delay" }
                    " (150ms). Moving on to a sibling waits the same delay, so the pointer can "
                    "cross one on its way into the submenu. There is no \"safe triangle\": "
                    "cut the corner slowly enough to rest on a sibling and it takes over."
                }
            },
            Demo {
                component: "Menu",
                children_text: "",
                fixed: vec!["state: menu".to_string(), "items".to_string()],
                code_child: Child(|_| TRIGGER.to_string()),
                wrap: Wrap(wrap),
                controls: vec![
                    Control::toggle("side", ["bottom", "top", "right", "left"])
                        .default("bottom")
                        .code(enum_code),
                    Control::toggle("align", ["start", "center", "end"])
                        .default("start")
                        .code(enum_code),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("close_on_select").default("true"),
                    Control::switch("loop_focus").default("true"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    MenuDemo {
                        side: values.str("side"),
                        align: values.str("align"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        close_on_select: values.str("close_on_select") == "true",
                        loop_focus: values.str("loop_focus") == "true",
                        disabled: values.str("disabled") == "true",
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "On the trigger, "
                    Kbd { "Enter" } " " Kbd { "Space" } " and " Kbd { "↓" }
                    " open the menu on its first item, " Kbd { "↑" } " on its last."
                }
                Text {
                    "In the menu, " Kbd { "↓" } " " Kbd { "↑" }
                    " move an item, wrapping unless "
                    Code { source: "loop_focus" }
                    " is off, and " Kbd { "Home" } " " Kbd { "End" } " go to the ends. "
                    Kbd { "Enter" } " and " Kbd { "Space" } " choose. "
                    Kbd { "→" } " opens a submenu and " Kbd { "←" } " closes it again. "
                    Kbd { "Esc" } " closes only the menu it is pressed in and hands focus back "
                    "to whatever opened it; " Kbd { "Tab" }
                    " closes every level and moves on from the trigger. Typing jumps to an "
                    "item: \"s\" to the next one starting with S, \"sav\" to Save, and a pause "
                    "of half a second starts over. A disabled item stays in the arrow order "
                    "but cannot be chosen."
                }
            }
        }
    }
}
