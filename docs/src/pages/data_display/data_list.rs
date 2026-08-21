use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Chip, Code, DataList, DataListItem, Text};

/// The pairs are the fixture - `orientation` and `gap` are the props - so the
/// code block prints them verbatim, `for` loop included: that loop is the
/// multi-description claim the lead makes.
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
            DocSection {
                title: "Usage",
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
}
