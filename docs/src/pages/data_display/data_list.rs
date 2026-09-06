use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Chip, Code, DataList, DataListItem, Text};

/// The pairs are the fixture - `orientation` and `gap` are the props - so the
/// code block prints them verbatim, `for` loop included: that loop is the
/// multi-description claim the lead makes.
// snippet: let phones = ["+49 30 1234567"];
// snippet: in DataList { .. }
const CHILDREN: &str = r#"DataListItem {
    label: rsx! { "Status" },
    Chip { variant: "filled", color: "success", size: "xs", "Active" }
}
DataListItem { label: rsx! { "Owner" }, "Jamie Chen" }
DataListItem {
    label: rsx! { "Phone" },
    for phone in &phones {
        "{phone}"
    }
}"#;

#[component]
pub fn DataListPage() -> Element {
    let phones = vec!["555-1234", "555-5678"];

    rsx! {
        DocPage {
            title: "DataList",
            source: "libero/src/components/data_display/data_list",
            markdown: "/md/data_list.md",
            properties: vec![
                props("DataList", vec![
                    prop("orientation", "Orientation")
                        .default("vertical")
                        .doc("`horizontal` puts each description beside its term; `vertical` stacks it below."),
                    prop("gap", "Size")
                        .default("md")
                        .doc("Row gap. Off-scale values go through `sx`."),
                    prop("children", "Element").doc("DataListItems, or any dt/dd content."),
                ]),
                props("DataListItem", vec![
                    prop("label", "Element").doc("The term (`<dt>`). `sx`/`class`/`states` decorate this element only - nothing wraps a term together with its descriptions."),
                    prop("children", "Element").doc("Descriptions for `label`. Under the dioxus-fork build this is `Vec<Element>`, so each child gets its own `<dd>`; against upstream main they collapse into one."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Renders a "
                    Code { source: "dl" }
                    " of term/description pairs. Unlike "
                    Code { source: "List" }
                    ", a term can have more than one description - "
                    Code { source: "DataListItem" }
                    "'s "
                    Code { source: "children" }
                    " is a "
                    Code { source: "Vec<Element>" }
                    ", not a single "
                    Code { source: "Element" }
                    ", so writing more than one child gives each one its own "
                    Code { source: "dd" }
                    " with no extra ceremony over a single description - including from a "
                    Code { source: "for" }
                    " loop, which flattens the same way."
                    // The split is a fork-only capability, so on main the page
                    // says what this build actually does instead.
                    if !cfg!(feature = "dioxus-fork") {
                        " Needs the "
                        Code { source: "dioxus-fork" }
                        " build - against upstream dioxus main the descriptions collapse into a single "
                        Code { source: "dd" }
                        "."
                    }
                }
            },
            // snippet: let phones = ["+49 30 1234567"];
            Demo {
                component: "DataList",
                children_text: "",
                children_code: CHILDREN,
                controls: vec![
                    Control::toggle("orientation", ["vertical", "horizontal"]),
                    Control::slider("gap", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                ],
                render: move |values: DemoValues| rsx! {
                    DataList {
                        orientation: values.str("orientation"),
                        gap: values.str("gap"),
                        DataListItem {
                            label: rsx! { "Status" },
                            Chip { variant: "filled", color: "success", size: "xs", "Active" }
                        }
                        DataListItem { label: rsx! { "Owner" }, "Jamie Chen" }
                        DataListItem {
                            label: rsx! { "Phone" },
                            for phone in phones.clone() {
                                "{phone}"
                            }
                        }
                    }
                },
            }
        }
    }
}
