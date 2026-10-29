use crate::components::{
    Child, Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Code, MenuPart, Text},
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

// From `sm` up: below it the menu is a sheet, and a docked width would shrink it to the trigger.
static DOCKED: &str = r#"sx: sx().breakpoint(
    Size::Sm,
    sx().width("var(--lsx-popover-anchor-width)")
        .max_width("none")
        .margin_top("var(--lsx-popover-gap)")
        .box_shadow("none"),
)"#;

/// A disabled menu disables its trigger too, so the trigger looks it. A docked
/// one fills the row it sits on.
fn trigger_code(values: &DemoValues) -> String {
    let mut trigger = TRIGGER.to_string();
    if values.str("docked") == "true" {
        trigger = trigger
            .replace("\"outlined\"", "\"tonal\"")
            .replace("    attributes:", "    radius: \"0\",\n    attributes:")
            .replace(
                "    \"Actions\"",
                "    sx: sx().width(\"100%\").height(\"52px\"),\n    \"Actions\"",
            );
    }
    match values.str("disabled").as_str() {
        "true" => trigger.replace("    attributes:", "    disabled: true,\n    attributes:"),
        _ => trigger,
    }
}

fn wrap(values: &DemoValues, source: &str) -> String {
    let layout = match values.str("docked").as_str() {
        "true" => {
            "direction: \"column\",\n        align: \"stretch\",\n        sx: sx().width(\"100%\"),"
        }
        _ => "direction: \"row\",\n        align: \"center\",",
    };
    format!(
        "{}\n\nrsx! {{\n    Flex {{\n        {layout}\n        gap: \"md\",\n{}        Text {{ size: \"sm\", role: \"status\", \"Last chosen: {{last()}}\" }}\n    }}\n}}",
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
                        .doc("Which side of the trigger the menu opens on. It flips when that side has no room. `Start`/`End` are logical: `Start` is the left under `dir=\"ltr\"`, the right under `rtl`."),
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
                        .doc("Called with ArrowLeft on the top level, or ArrowRight on an item without a submenu. `Menubar` uses it to move to the next menu."),
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
                        .doc("Opens a second menu beside the item instead, on Right, a click, or when the pointer rests on the item for 150ms. Moving to a sibling waits as long, so the pointer can cross one on its way in. An item runs a command or opens a submenu, and the later call wins."),
                    prop("href", "String")
                        .doc("Makes the item a link: an `<a>` that opens the URL in a new tab, so middle-click and the context menu work. Space activates it like Enter. Replaces `onselect` and `submenu`. A disabled link has no `href`. The label ends in a hidden \"(opens in a new tab)\", as on `Anchor`."),
                    prop("new_tab_hint", "bool")
                        .default("true")
                        .doc("With `href`: `false` drops the hidden hint and the new-tab icon."),
                    prop("new_tab_icon", "bool")
                        .default(if theme.anchor.new_tab_icon { "true" } else { "false" })
                        .doc("With `href`: draws the new-tab icon after the label. Unset, the theme's `anchor.new_tab_icon`, which is off."),
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
                    prop("color", "ThemeAwareValue")
                        .doc("Colours the label and `leading`, such as `\"error\"` for a destructive command. A palette colour takes its readable text shade. The trailing content, shortcut and check keep the menu's colour."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Enter", "Space", "Down"], "On the trigger: opens the menu on the checked radio item, else its first enabled item.")
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
                    "`menu.a11y_attributes()` wires your trigger. `menu.a11y_attributes_on(attributes)` wires a trigger that already has an `id`, such as a `FileField` trigger's: the menu keeps that id and takes the trigger's name.",
                    "Below the `sm` breakpoint the menu opens as a sheet at the foot of the screen, with the same items. A swipe down from its top, a press outside, Escape and Back close it. A submenu opens as a second sheet over it and closes alone.",
                    "Android's Back button closes the menu as Escape does, rather than the app.",
                    "A tap or a pen opens the menu on the menu itself, with no item highlighted. Down or Up then moves to the first or last item.",
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
                    " ancestor clips it. Its position and width are class rules, so "
                    Code { source: "sx" }
                    " can dock it: "
                    Code { source: "var(--lsx-popover-anchor-width)" }
                    " is the trigger's width. Set it from the "
                    Code { source: "sm" }
                    " breakpoint up with "
                    Code { source: "sx().breakpoint(Size::Sm, ..)" }
                    ": below it the menu is a full-width sheet."
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
                    // Docks the menu on the trigger's edge, as wide as the trigger; best with side `top`.
                    Control::switch("docked").default("true").code(|_, values| match values.str("docked").as_str() {
                        "true" => vec![DOCKED.to_string()],
                        _ => vec![],
                    }),
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
                        docked: values.str("docked") == "true",
                    }
                },
            }
        }
    }
}
