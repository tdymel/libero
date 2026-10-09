use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, MenubarPart, Text};
use libero::use_theme;

mod demo;
use demo::MenubarDemo;

/// The live demo; its state, pick handler and menus print above the `rsx!`.
const FILE: DemoFile = DemoFile(include_str!("menubar/demo.rs"));

fn wrap(_: &DemoValues, source: &str) -> String {
    format!(
        "{}\n\nrsx! {{\n    Flex {{\n        direction: \"column\",\n        gap: \"md\",\n{}        Text {{ size: \"sm\", role: \"status\", \"Last chosen: {{last()}}\" }}\n    }}\n}}",
        FILE.section("preamble"),
        indent(&indent(source))
    )
}

#[component]
pub fn MenubarPage() -> Element {
    let theme = use_theme();
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
                        .default(theme.menubar.size.as_str())
                        .doc("The triggers' font and padding, and each menu's item size."),
                    prop("radius", "ThemeAwareValue")
                        .default(theme.menubar.radius.as_str())
                        .doc("The triggers' and the menus' corner radius, or any CSS, e.g. `radius: \"0\"`."),
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
                .must(["Name the bar with `aria_label`. It is required."])
                .example("An editor's menu bar, `Menubar { aria_label: \"Editor\", .. }` with File, Edit and View: Tab enters on File, Right moves to Edit, and Down opens the Edit menu on its first item."),
            lead: rsx! {
                Text {
                    "A row of menus, like a desktop app's File, Edit and View. Each menu is a "
                    Code { source: "Menu" }
                    " with the same items. The bar keeps one menu open at most and is a single "
                    "tab stop. Click a trigger to open its menu. While one is open, hovering "
                    "another trigger switches to it."
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
                    Control::sizes("size").default(theme.menubar.size.as_str()),
                    Control::sizes("radius").default(theme.menubar.radius.as_str()),
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
                title: "When to use it",
                Text {
                    "Use it in an app such as an editor. For page navigation, use links. For "
                    "one set of actions, use a single "
                    Code { source: "Menu" }
                    "."
                }
            }
        }
    }
}
