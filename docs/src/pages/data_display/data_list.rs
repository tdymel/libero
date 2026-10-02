use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Chip, Code, DataList, DataListItem, Text};
use libero::use_theme;

/// The pairs, printed verbatim. The `for` loop shows several descriptions per term.
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
        div { "{phone}" }
    }
}"#;

#[component]
pub fn DataListPage() -> Element {
    let theme = use_theme();
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
                        .doc("`horizontal` puts each description beside its term, `vertical` below it."),
                    prop("gap", "Size")
                        .default(theme.data_list.size.as_str())
                        .doc("Row gap. Other values go through `sx`."),
                    prop("children", "Element")
                        .default("required")
                        .doc("`DataListItem`s, or any `dt` and `dd` content."),
                ]),
                props("DataListItem", vec![
                    prop("label", "Element")
                        .default("required")
                        .doc("The term, a `<dt>`. `sx`, `class` and `states` style the term only."),
                    prop("children", "Element")
                        .default("required")
                        .doc("The term's descriptions, in one `<dd>`."),
                ]),
            ],
            accessibility: a11y()
                .handles(["Each term is a `<dt>` and its descriptions one `<dd>`, so a screen reader pairs them."])
                .must([
                    "Put each `DataListItem` directly inside the `DataList`. A wrapper element between them breaks the pairing of term and description.",
                ]),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "dl" }
                    " of terms and descriptions. One term can carry several descriptions, "
                    "which share one "
                    Code { source: "dd" }
                    "."
                }
            },
            // snippet: let phones = ["+49 30 1234567"];
            Demo {
                component: "DataList",
                children_text: "",
                children_code: CHILDREN,
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical"),
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
                                div { "{phone}" }
                            }
                        }
                    }
                },
            }
        }
    }
}
