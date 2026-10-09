use crate::components::{
    Child, Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Wrap, a11y, indent, prop,
    props,
};
use dioxus::prelude::*;
use libero::{
    components::{Code, Kbd, MenuPart, Text},
    use_theme,
};

mod demo;
use demo::MenuDemo;

/// The live demo; its state, pick handler and items print above the `rsx!`.
const FILE: DemoFile = DemoFile(include_str!("menu/demo.rs"));

// `static`: the snippet scan compiles it in the Demo's output, not alone.
static TRIGGER: &str = r#"Button {
    variant: "outlined",
    attributes: menu.a11y_attributes(),
    "Actions"
}"#;

/// A disabled menu disables its trigger too, so the trigger looks it.
fn trigger_code(values: &DemoValues) -> String {
    match values.str("disabled").as_str() {
        "true" => TRIGGER.replace("    attributes:", "    disabled: true,\n    attributes:"),
        _ => TRIGGER.to_string(),
    }
}

fn wrap(_: &DemoValues, source: &str) -> String {
    format!(
        "{}\n\nrsx! {{\n    Flex {{\n        direction: \"row\",\n        align: \"center\",\n        gap: \"md\",\n{}        Text {{ size: \"sm\", role: \"status\", \"Last chosen: {{last()}}\" }}\n    }}\n}}",
        FILE.section("preamble"),
        indent(&indent(source))
    )
}

#[component]
pub fn MenuPage() -> Element {
    let theme = use_theme();

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
                        .default(theme.menu.close_on_select.to_string())
                        .doc("Whether choosing an item closes the menu."),
                    prop("loop_focus", "bool")
                        .default(theme.menu.loop_focus.to_string())
                        .doc("Whether the arrow keys wrap from the last item to the first."),
                    prop("size", "Size")
                        .default(theme.menu.size.as_str())
                        .doc("Item height and font size."),
                    prop("radius", "ThemeAwareValue")
                        .default(theme.menu.radius.as_str())
                        .doc("The menu's corner radius, or any CSS, e.g. `radius: \"0\"`. The items' corners follow it."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("The trigger opens nothing, and an open menu closes. Disable the trigger too, so it looks disabled."),
                    prop("onedge", "Callback<MenuEdge>")
                        .doc("Called with ← on the top level, or → on an item without a submenu. `Menubar` uses it to move to the next menu."),
                    prop("parts", "Parts<MenuPart>")
                        .doc("Styles for the inner parts in the Style API tab, on every menu level, submenus too, as `sx` does: `Parts::new().part(MenuPart::Label, sx().font_weight(\"500\"))`."),
                ])
                .parts("MenuPart", vec![
                    (MenuPart::Item, "A row: `menuitem`, `menuitemradio` or `menuitemcheckbox`."),
                    (MenuPart::GroupLabel, "A group's visible name."),
                    (MenuPart::Check, "The check column, on every row of a level with a checkable item."),
                    (MenuPart::Leading, "The item's `leading` content."),
                    (MenuPart::Label, "The item's text."),
                    (MenuPart::Trailing, "The item's `trailing` content."),
                    (MenuPart::Shortcut, "The key hint, hidden from screen readers."),
                    (MenuPart::Chevron, "A submenu item's arrow."),
                ]),
                props("MenuItem", vec![
                    prop("new(label)", "String")
                        .doc("The visible text, the accessible name, and what typeahead matches."),
                    prop("onselect", "FnMut(())")
                        .doc("Runs when the item is chosen by a click, Enter or Space."),
                    prop("submenu", "Vec<MenuEntry>")
                        .doc("Opens a second menu beside the item instead. An item runs a command or opens a submenu, and the later call wins."),
                    prop("href", "String")
                        .doc("Makes the item a link: an `<a>` that opens the URL in a new tab, so middle-click and the context menu work. Replaces `onselect` and `submenu`. A disabled link has no `href`. The label ends in a new-tab icon and a hidden \"(opens in a new tab)\", as on `Anchor`."),
                    prop("new_tab_hint", "bool")
                        .default("true")
                        .doc("With `href`: `false` drops the new-tab icon and the hidden hint."),
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
                    prop("description", "String")
                        .doc("Read after the label through `aria-describedby`, never shown. Say why a `disabled` item cannot be chosen."),
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
                    "Android's Back button closes the menu as Escape does, rather than the app.",
                ])
                .must([
                    "Put a shortcut hint in `shortcut`, not `trailing`. A screen reader then hears it as `aria-keyshortcuts`, not as part of the item's name.",
                ])
                .limits(["On Android, a menu opened without a tap (on mount or from a timer) may let Back close the app."])
                .example("An \"Actions\" menu with a Save item, `shortcut: \"Ctrl+S\"`: Enter on the trigger opens the menu on its first item, a screen reader reads \"Save\" and the shortcut apart, and Escape returns focus to the trigger."),
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
            },
            Demo {
                component: "Menu",
                children_text: "",
                fixed: vec!["state: menu".to_string(), "items".to_string()],
                code_child: Child(trigger_code),
                wrap: Wrap(wrap),
                controls: vec![
                    Control::side(["top", "end", "bottom", "start"]),
                    Control::align(),
                    Control::sizes("size")
                        .default(theme.menu.size.as_str()),
                    Control::sizes("radius")
                        .default(theme.menu.radius.as_str()),
                    Control::switch("close_on_select").default(theme.menu.close_on_select.to_string()),
                    Control::switch("loop_focus").default(theme.menu.loop_focus.to_string()),
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
                title: "Submenus",
                Text {
                    "A submenu opens beside its item on "
                    Kbd { "→" }
                    ", a click, or when the pointer rests on the item for 150ms. Moving to a "
                    "sibling waits as long, so the pointer can cross one on its way into the "
                    "submenu."
                }
            }

            DocSection {
                title: "Links",
                Text {
                    "An item can be a link: "
                    Code { source: "MenuItem::new(\"Docs\").href(url)" }
                    " renders an "
                    Code { source: "<a>" }
                    " that opens "
                    Code { source: "url" }
                    " in a new tab. Space activates it like Enter."
                }
            }
        }
    }
}
