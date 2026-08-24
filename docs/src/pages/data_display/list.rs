use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, List, ListItem, Text};

/// The items are the fixture here - `size` is the only prop - so the code
/// block prints them verbatim. Only the outer list carries `size`: nested
/// indent comes from the *parent*'s `& ul` rule.
const CHILDREN: &str = r#"ListItem { "First item" }
ListItem { "Second item" }
ListItem {
    "Third item, with a nested list"
    List {
        ListItem { "Nested one" }
        ListItem { "Nested two" }
    }
}"#;

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
                    prop("children", "Element").doc("The list's items."),
                ]),
                props("ListItem", vec![
                    prop("children", "Element").doc("The item's content."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Renders a "
                    Code { source: "ul" }
                    "/"
                    Code { source: "li" }
                    " pair with the browser's default list styling removed - nested lists "
                    "indent relative to their own content. "
                    Code { source: "size" }
                    " (xs-xxl, default "
                    Code { source: "md" }
                    ") controls item gap and nested-list indent together."
                }
            },
            Demo {
                component: "List",
                children_text: "",
                children_code: CHILDREN,
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                ],
                render: move |values: DemoValues| rsx! {
                    List {
                        size: values.str("size"),
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
