use crate::components::{Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Chip, Code, Table, Text, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    role: String,
    bonus: Option<f64>,
}

fn people() -> Vec<Person> {
    vec![
        Person {
            name: "Ada Lovelace".into(),
            role: "Owner".into(),
            bonus: Some(12.5),
        },
        Person {
            name: "Grace Hopper".into(),
            role: "Admin".into(),
            bonus: Some(8.0),
        },
        Person {
            name: "Alan Turing".into(),
            role: "Viewer".into(),
            bonus: None,
        },
    ]
}

// snippet: item #[derive(Clone, PartialEq)] struct Person { name: String, role: String, bonus: f64 }
// snippet: let people: Vec<Person> = Vec::new();
// snippet: in Table { aria_label: "Team members", data: people, .. }
const COLUMNS: &str = r#"columns: vec![
        column("Name").value(|p: &Person| p.name.clone()).sortable(),
        column("Role")
            .value(|p: &Person| p.role.clone())
            .sortable()
            .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
        column("Bonus").value(|p: &Person| p.bonus).sortable(),
    ]"#;

/// The rows are the fixture, and the snippet only compiles with them, so the
/// code block carries the struct and the data above the `Table` itself.
fn wrap_data(_: &DemoValues, code: &str) -> String {
    format!(
        r#"#[derive(Clone, PartialEq)]
struct Person {{
    name: String,
    role: String,
    bonus: Option<f64>,
}}

let people = vec![
    Person {{ name: "Ada Lovelace".into(), role: "Owner".into(), bonus: Some(12.5) }},
    Person {{ name: "Grace Hopper".into(), role: "Admin".into(), bonus: Some(8.0) }},
    Person {{ name: "Alan Turing".into(), role: "Viewer".into(), bonus: None }},
];

{code}"#
    )
}

#[component]
pub fn TablePage() -> Element {
    rsx! {
        DocPage {
            title: "Table",
            source: "libero/src/components/data_display/table",
            markdown: "/md/table.md",
            properties: vec![
                props("Table", vec![
                    prop("data", "Vec<T>").doc("One row each, in source order until a column is sorted."),
                    prop("columns", "Vec<Column<T>>").doc("Built with `column(..)`."),
                ]),
                props("column()", vec![
                    prop("header", "String").doc("The column's title, given as the argument to `column(..)`."),
                    prop("value", "fn(&T) -> V").doc("Reads one cell out of a row. `V`'s `CellValue` impl decides sort order and alignment, then is erased. Strings sort as text and align left, every integer and float sorts numerically and aligns right, `bool` prints `true`/`false`, and `Option<V>` keeps the inner alignment, renders `None` empty and sorts it last in both directions. Your own type joins them with one `impl CellValue`: `cell_text` is required, `sort_key` and `align` have defaults."),
                    prop("sortable", "bool").default("false").doc("Turns the header into a sort button."),
                    prop("render", "fn(&T) -> Element").doc("Replaces the cell body. Sorting still uses `value`."),
                    prop("align", "CellAlign").default("follows the cell type").doc("Overrides the alignment `value`'s type chose."),
                ]).without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "Data in, table out. Each column is built with "
                    Code { source: "column" }
                    " - a header, a "
                    Code { source: "value" }
                    " that reads one cell out of a row, and whatever else that column needs. "
                    Code { source: "Table" }
                    " itself takes only "
                    Code { source: "data" }
                    " and "
                    Code { source: "columns" }
                    "."
                }
                Text {
                    "The cell's type does the quiet work: "
                    Code { source: "value" }
                    " reads it for the column's sort order and alignment, then erases it, which is why columns over different cell types live in one "
                    Code { source: "Vec" }
                    ". A numeric column sorts numerically and aligns right without being told."
                }
                Text {
                    Code { source: "sortable" }
                    " turns a header into a button - the first click sorts ascending, the next flips it, and the sort state stays inside "
                    Code { source: "Table" }
                    ". "
                    Code { source: "render" }
                    " changes only what a cell draws, so the Role column below still sorts by its text and not by its chip."
                }
            },
            Demo {
                component: "Table",
                children_text: "",
                fixed: vec![
                    r#"aria_label: "Team members""#.to_string(),
                    "data: people".to_string(),
                    COLUMNS.to_string(),
                ],
                controls: vec![],
                wrap: Wrap(wrap_data),
                render: move |_: DemoValues| rsx! {
                    Table {
                        aria_label: "Team members",
                        data: people(),
                        columns: vec![
                            column("Name").value(|p: &Person| p.name.clone()).sortable(),
                            column("Role")
                                .value(|p: &Person| p.role.clone())
                                .sortable()
                                .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
                            column("Bonus").value(|p: &Person| p.bonus).sortable(),
                        ],
                    }
                },
            }
        }
    }
}
