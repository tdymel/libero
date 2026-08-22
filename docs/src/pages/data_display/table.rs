use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap};
use dioxus::prelude::*;
use libero::components::{Chip, Code, CodeBlock, Table, Text, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    role: String,
    age: u32,
    bonus: Option<f64>,
}

fn people() -> Vec<Person> {
    vec![
        Person {
            name: "Ada Lovelace".into(),
            role: "Owner".into(),
            age: 36,
            bonus: Some(12.5),
        },
        Person {
            name: "Grace Hopper".into(),
            role: "Admin".into(),
            age: 45,
            bonus: Some(8.0),
        },
        Person {
            name: "Alan Turing".into(),
            role: "Viewer".into(),
            age: 9,
            bonus: None,
        },
    ]
}

const COLUMNS: &str = r#"columns: vec![
        column("Name").value(|p: &Person| p.name.clone()).sortable(),
        column("Role")
            .value(|p: &Person| p.role.clone())
            .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
        column("Age").value(|p: &Person| p.age).sortable(),
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
    age: u32,
    bonus: Option<f64>,
}}

let people = vec![
    Person {{ name: "Ada Lovelace".into(), role: "Owner".into(), age: 36, bonus: Some(12.5) }},
    Person {{ name: "Grace Hopper".into(), role: "Admin".into(), age: 45, bonus: Some(8.0) }},
    Person {{ name: "Alan Turing".into(), role: "Viewer".into(), age: 9, bonus: None }},
];

{code}"#
    )
}

const CELL_TYPES: &str = r#"|p: &Person| p.name.clone()  // String      -> text sort, start-aligned
|p: &Person| p.age          // u32         -> numeric sort, end-aligned
|p: &Person| p.bonus        // Option<f64> -> None renders empty, sorts last"#;

#[component]
pub fn TablePage() -> Element {
    rsx! {
        DocPage {
            title: "Table",
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
            DocSection {
                title: "Usage",
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
                                    .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
                                column("Age").value(|p: &Person| p.age).sortable(),
                                column("Bonus").value(|p: &Person| p.bonus).sortable(),
                            ],
                        }
                    },
                }
            }
            DocSection {
                title: "The cell type decides",
                Text {
                    "Strings sort as text and align left. Every integer and float sorts numerically and aligns right. "
                    Code { source: "bool" }
                    " prints "
                    Code { source: "true" }
                    "/"
                    Code { source: "false" }
                    ". "
                    Code { source: "Option<V>" }
                    " keeps the inner type's alignment, renders "
                    Code { source: "None" }
                    " as empty, and sorts it last in both directions."
                }
                CodeBlock { source: CELL_TYPES, language: "rust" }
                Text {
                    "Your own type joins them with one "
                    Code { source: "impl CellValue" }
                    ": "
                    Code { source: "cell_text" }
                    " is required, "
                    Code { source: "sort_key" }
                    " and "
                    Code { source: "align" }
                    " have defaults."
                }
            }
        }
    }
}
