use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Kbd, Menu, MenuEntry, MenuItem, Text, use_menu},
    hooks::{Align, Side},
};

/// Everything above the `rsx!`, the state, the pick handler and the items.
/// `generate_code` only emits props, and the list is a `let`.
const PREAMBLE: &str = r#"let menu = use_menu();
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

"#;

// snippet: after PREAMBLE
const TRIGGER: &str = r#"Button {
    variant: "outlined",
    attributes: menu.a11y_attributes(),
    "Actions"
}"#;

fn wrap(_: &DemoValues, source: &str) -> String {
    format!(
        "{PREAMBLE}rsx! {{\n    Flex {{\n        direction: \"row\",\n        align: \"center\",\n        gap: \"md\",\n{}        Text {{ size: \"sm\", \"Last chosen: {{last()}}\" }}\n    }}\n}}",
        indent(&indent(source))
    )
}

fn side_of(value: &str) -> Side {
    match value {
        "top" => Side::Top,
        "start" => Side::Start,
        "end" => Side::End,
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
            source: "libero/src/components/overlay/menu",
            markdown: "/md/menu.md",
            properties: vec![
                props("Menu", vec![
                    prop("state", "MenuState")
                        .default("required")
                        .doc("From `use_menu()`. Holds the open state and wires the trigger to the menu."),
                    prop("items", "Vec<MenuEntry>")
                        .default("required")
                        .doc("The menu, in order. `MenuEntry::Item`, `MenuEntry::Group { label, items }` for a named section, and `MenuEntry::Separator`."),
                    prop("children", "Element")
                        .default("required")
                        .doc("The trigger, carrying `menu.a11y_attributes()`. It needs no click or key handler of its own."),
                    prop("side", "Side")
                        .default("Bottom")
                        .doc("Which side of the trigger the menu opens on. It flips when that side has no room."),
                    prop("align", "Align")
                        .default("Start")
                        .doc("Where the menu lines up along that side."),
                    prop("close_on_select", "bool")
                        .default("true")
                        .doc("Whether choosing an item closes the menu."),
                    prop("loop_focus", "bool")
                        .default("true")
                        .doc("Whether the arrow keys wrap from the last item to the first."),
                    prop("size", "Size")
                        .default("md")
                        .doc("Item height and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("The menu's corner radius. The items' corners follow it."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("The trigger opens nothing, and an open menu closes. Disable the trigger too, so it looks disabled."),
                    prop("onedge", "Callback<MenuEdge>")
                        .doc("Called with ← on the top level, or → on an item without a submenu. `Menubar` uses it to move to the next menu."),
                ]),
                props("MenuItem", vec![
                    prop("new(label)", "String")
                        .doc("The visible text, the accessible name, and what typeahead matches."),
                    prop("onselect", "FnMut(())")
                        .doc("Runs when the item is chosen by a click, Enter or Space."),
                    prop("submenu", "Vec<MenuEntry>")
                        .doc("Opens a second menu beside the item instead. An item runs a command or opens a submenu, and the later call wins."),
                    prop("href", "String")
                        .doc("Makes the item a link: an `<a>` that opens the URL in a new tab, so middle-click and the context menu work. Replaces `onselect` and `submenu`. A disabled link has no `href`."),
                    prop("keep_open", "()")
                        .doc("Choosing the item leaves the menu open, and focus stays on it, whatever the menu's `close_on_select` says. `close_on_select(bool)` overrides it either way."),
                    prop("leading", "Element")
                        .doc("Before the label, such as an icon. Nothing interactive, since it sits inside the item's button."),
                    prop("trailing", "Element")
                        .doc("At the far end, such as a badge. It joins the accessible name. Nothing interactive."),
                    prop("shortcut", "&str")
                        .doc("The key that runs the item outside the menu, in `aria-keyshortcuts` syntax (`\"Control+X\"`). Drawn as a hint (\"Ctrl+X\") and kept out of the name. `Control`, `Shift`, `Alt` and `Meta` are drawn in the localization's `menu` words (\"Strg+Umschalt+S\" in German). You bind the key yourself."),
                    prop("radio", "bool")
                        .doc("Makes the item one choice of several, with a check while `true`. Put the choices in one `Group` and keep one checked. A menu opens on its checked item."),
                    prop("checkbox", "bool")
                        .doc("Makes the item an on/off setting, with a check while `true`. Flip it in `onselect`. An item is `radio` or `checkbox`, and the later call wins."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Stays in the arrow-key order but cannot be chosen, and typeahead skips it."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Enter", "Space", "Down"], "On the trigger: opens the menu on its first item.")
                .key(["Up"], "On the trigger: opens the menu on its last item.")
                .key(["Down", "Up"], "In the menu: moves an item, wrapping unless `loop_focus` is off.")
                .key(["Home", "End"], "In the menu: goes to the first or last item.")
                .key(["Enter", "Space"], "In the menu: chooses the item.")
                .key(["Right"], "Opens a submenu.")
                .key(["Left"], "Closes a submenu again.")
                .key(["Escape"], "Closes only the menu it is pressed in and returns focus to what opened it.")
                .key(["Tab"], "Closes every level and moves on from the trigger.")
                .handles([
                    "Typing jumps to an item, \"s\" to the next one starting with S and \"sav\" to Save. A pause of half a second starts over.",
                    "`menu.a11y_attributes()` wires your trigger.",
                ])
                .must([
                    "Put a shortcut hint in `shortcut`, not `trailing`. A screen reader then hears it as `aria-keyshortcuts`, not as part of the item's name.",
                ]),
            lead: rsx! {
                Text {
                    "A list of commands that drops from a trigger. The items are data, not "
                    "children. "
                    Code { source: "use_menu()" }
                    " keeps the open state in your scope, and the trigger is your own "
                    Code { source: "Button" }
                    ", wired by "
                    Code { source: "menu.a11y_attributes()" }
                    ". The menu is portaled, so no "
                    Code { source: "overflow: hidden" }
                    " ancestor clips it."
                }
                Text {
                    "A submenu opens beside its item on "
                    Kbd { "→" }
                    ", a click, or when the pointer rests on the item for 150ms. Moving to a "
                    "sibling waits as long, so the pointer can cross one on its way into the "
                    "submenu."
                }
                Text {
                    "An item can be a link: "
                    Code { source: "MenuItem::new(\"Docs\").href(url)" }
                    " renders an "
                    Code { source: "<a>" }
                    " that opens "
                    Code { source: "url" }
                    " in a new tab. Space activates it like Enter."
                }
            },
            Demo {
                component: "Menu",
                children_text: "",
                fixed: vec!["state: menu".to_string(), "items".to_string()],
                code_child: Child(|_| TRIGGER.to_string()),
                wrap: Wrap(wrap),
                controls: vec![
                    Control::toggle("side", ["top", "end", "bottom", "start"])
                        .labels(["Top", "End", "Bottom", "Start"])
                        .default("bottom")
                        .code(enum_code),
                    Control::toggle("align", ["start", "center", "end"])
                        .labels(["Start", "Center", "End"])
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
        }
    }
}
