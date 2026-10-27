use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, a11y, prop, props};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::components::{Button, Code, Icon, List, ListItem, ListItemPart, ListSubheader, Text};
use libero::use_theme;

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("list.rs"));

// snippet: in List { .. }
const ICON_CODE: &str = r#"icon: rsx! { Icon { variant: "standard", color: "primary", size: "sm", Pictogram { icon: lucide::check::outlined } } }"#;

#[component]
pub fn ListPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "List",
            source: "libero/src/components/data_display/list",
            markdown: "/md/list.md",
            properties: vec![
                props("List", vec![
                    prop("size", "Size")
                        .default(theme.list.size.as_str())
                        .doc("Item gap and nested-list indent, together. A nested ordered list keeps the fixed `2em` its numbers need."),
                    prop("ordered", "bool")
                        .default("false")
                        .doc("An `ol` with visible numbers, for items whose order matters."),
                    prop("icon", "Option<Element>")
                        .default("None")
                        .doc("Shown at the start of every item, beside its first line. Hidden from screen readers. A nested list does not take its parent's icon. In an ordered list an icon replaces its item's number and the gutter beside it."),
                    prop("children", "Element").default("required").doc("The list's items."),
                ]),
                props("ListItem", vec![
                    prop("icon", "Option<Element>")
                        .default("None")
                        .doc("This item's own icon, in place of the list's."),
                    prop("action", "Option<Element>")
                        .default("None")
                        .doc("A control at the item's end, such as a button, beside the content. Keep it out of `children`."),
                    prop("parts", "Parts<ListItemPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element")
                        .default("required")
                        .doc("The item's content. Keep the list's font size: the icon gutter is in `em` of the list, so a `font_size` on one item misaligns its icon."),
                ])
                .parts("ListItemPart", vec![
                    (ListItemPart::Icon, "The icon wrapper, with an icon only."),
                    (ListItemPart::Body, "The content beside the icon and the action, with either."),
                    (ListItemPart::Action, "The trailing `action`, with one only."),
                ]),
                props("ListSubheader", vec![
                    prop("children", "Element")
                        .default("required")
                        .doc("The caption. An ordered list does not number it, but a screen reader counts it: use one ordered `List` per group instead."),
                ]),
            ],
            accessibility: a11y()
                .handles(["Icons are hidden from screen readers."])
                .must([
                    "Keep a `List`'s children to `ListItem`s and `ListSubheader`s. A stray element between them breaks the list and its item count for a screen reader.",
                    "When an icon carries meaning, such as done or missing, say it in the item's text too.",
                    "A `ListSubheader` is read as one more item. For groups a screen reader names, nest one `List` per group, each with an `aria-label`.",
                    "In an ordered `List`, never use a `ListSubheader`: the numbers skip it but a screen reader counts it, so \"1. First item\" is announced as item 2 of 4. Nest one ordered `List` per group instead; a debug build warns.",
                    "Name each repeated `action` with its item, such as `aria-label: \"Edit Second item\"`, when several items share a label. Every button would otherwise read \"Edit\".",
                ])
                .example("A checklist of `ListItem`s with check icons: a screen reader says the item count and reads each item's text. The item says \"done\" in words, since the icon is not read."),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "ul" }
                    " of "
                    Code { source: "li" }
                    " items without the browser's list styling. A nested list indents from "
                    "its own content. Set "
                    Code { source: "size" }
                    " on the outer list only, since the parent sets a nested list's indent. "
                    "A nested ordered list keeps a fixed 2em, room for its numbers. A "
                    Code { source: "ListSubheader" }
                    " captions the items after it."
                }
            },
            Demo {
                component: "List",
                children_text: "",
                children_code: FILE.section("children"),
                controls: vec![
                    Control::sizes("size")
                        .default(theme.list.size.as_str()),
                    Control::switch("ordered"),
                    Control::switch("icon").code(|_, values| {
                        match values.str("icon").as_str() {
                            "true" => vec![ICON_CODE.to_string()],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    List {
                        size: values.str("size"),
                        ordered: values.str("ordered") == "true",
                        icon: (values.str("icon") == "true").then(|| rsx! {
                            Icon { variant: "standard", color: "primary", size: "sm", Pictogram { icon: lucide::check::outlined } }
                        }),
                        // Not in the code: an ordered list is counted wrong with one (see `must`).
                        if values.str("ordered") != "true" {
                            ListSubheader { "Getting started" }
                        }
                        // demo-code: children start
                        ListItem { "First item" }
                        ListItem {
                            action: rsx! { Button { size: "sm", variant: "outlined", "aria-label": "Edit Second item", "Edit" } },
                            "Second item"
                        }
                        ListItem {
                            "Third item, with a nested list"
                            List {
                                ListItem { "Nested one" }
                                ListItem { "Nested two" }
                            }
                        }
                        // demo-code: children end
                    }
                },
            }
        }
    }
}
