use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::components::{Code, Icon, List, ListItem, ListItemPart, Text};

/// The items, printed verbatim. Only the outer list carries `size`: nested indent comes from
/// the parent's `& ul` rule.
const CHILDREN: &str = r#"ListItem { "First item" }
ListItem { "Second item" }
ListItem {
    "Third item, with a nested list"
    List {
        ListItem { "Nested one" }
        ListItem { "Nested two" }
    }
}"#;

// snippet: in List { .. }
const ICON_CODE: &str = r#"icon: rsx! { Icon { variant: "standard", color: "primary", size: "sm", Pictogram { icon: lucide::check::outlined } } }"#;

#[component]
pub fn ListPage() -> Element {
    rsx! {
        DocPage {
            title: "List",
            source: "libero/src/components/data_display/list",
            markdown: "/md/list.md",
            properties: vec![
                props("List", vec![
                    prop("size", "Size")
                        .default("md")
                        .doc("Item gap and nested-list indent, together."),
                    prop("ordered", "bool")
                        .default("false")
                        .doc("An `ol` with visible numbers, for items whose order matters."),
                    prop("icon", "Option<Element>")
                        .default("None")
                        .doc("Shown at the start of every item, beside its first line. Hidden from screen readers."),
                    prop("children", "Element").default("required").doc("The list's items."),
                ]),
                props("ListItem", vec![
                    prop("icon", "Option<Element>")
                        .default("None")
                        .doc("This item's own icon, in place of the list's."),
                    prop("parts", "Parts<ListItemPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").default("required").doc("The item's content."),
                ])
                .parts("ListItemPart", vec![
                    (ListItemPart::Icon, "The icon wrapper, with an icon only."),
                    (ListItemPart::Body, "The content beside the icon, with an icon only."),
                ]),
            ],
            accessibility: a11y()
                .handles(["Icons are hidden from screen readers."])
                .must([
                    "Keep a `List`'s children to `ListItem`s. A stray element between them breaks the list and its item count for a screen reader.",
                    "When an icon carries meaning, such as done or missing, say it in the item's text too.",
                ]),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "ul" }
                    " of "
                    Code { source: "li" }
                    " items without the browser's list styling. A nested list indents from "
                    "its own content. Set "
                    Code { source: "size" }
                    " on the outer list only, since the parent sets a nested list's indent."
                }
                Text {
                    "An "
                    Code { source: "icon" }
                    " on the list marks every item. A "
                    Code { source: "ListItem" }
                    "'s own "
                    Code { source: "icon" }
                    " replaces it for that item. A nested list does not take its parent's icon."
                }
            },
            Demo {
                component: "List",
                children_text: "",
                children_code: CHILDREN,
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
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
                        ListItem { "First item" }
                        ListItem { "Second item" }
                        ListItem {
                            "Third item, with a nested list"
                            List {
                                ListItem { "Nested one" }
                                ListItem { "Nested two" }
                            }
                        }
                    }
                },
            }
        }
    }
}
