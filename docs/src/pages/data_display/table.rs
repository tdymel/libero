use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
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
// snippet: in Table { caption: "Team members", data: people, .. }
const COLUMNS: &str = r#"columns: vec![
        column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
        column("Role")
            .value(|p: &Person| p.role.clone())
            .sortable()
            .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
        column("Bonus").value(|p: &Person| p.bonus).sortable(),
    ]"#;

/// The rows are the fixture, and the snippet only compiles with them, so the
/// code block carries the struct and the data above the `Table` itself.
fn wrap_data(values: &DemoValues, code: &str) -> String {
    let people = match no_rows(values) {
        true => "let people: Vec<Person> = Vec::new();",
        false => {
            r#"let people = vec![
    Person { name: "Ada Lovelace".into(), role: "Owner".into(), bonus: Some(12.5) },
    Person { name: "Grace Hopper".into(), role: "Admin".into(), bonus: Some(8.0) },
    Person { name: "Alan Turing".into(), role: "Viewer".into(), bonus: None },
];"#
        }
    };
    format!(
        r#"#[derive(Clone, PartialEq)]
struct Person {{
    name: String,
    role: String,
    bonus: Option<f64>,
}}

{people}

{code}"#
    )
}

/// The `empty` switch empties `data` too, or the slot would never show.
fn no_rows(values: &DemoValues) -> bool {
    values.str("empty") == "true"
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
                    prop("data", "Vec<T>").default("required").doc("One row each, in source order until a column is sorted."),
                    prop("columns", "Vec<Column<T>>").default("required").doc("Built with `column(..)`."),
                    prop("caption", "Option<String>").default("None").doc("A visible title above the header row, and the table's accessible name."),
                    prop("empty", "Option<Element>").default("None").doc("Shown in one full-width row when `data` is empty."),
                    prop("scroll", "bool").default("false").doc("Wraps the table in a named, focusable region that scrolls sideways. `class`, `sx` and `attributes` stay on the table."),
                ]),
                props("column()", vec![
                    prop("header", "String").default("required").doc("The column's title, the argument to `column(..)`."),
                    prop("value", "fn(&T) -> V").default("required").doc("Reads one cell out of a row. `V` sets the sort order and alignment. Text sorts as text and aligns left, numbers sort numerically and align right, and `Option<V>` renders `None` empty and sorts it last. Your own type joins them with one `impl CellValue`."),
                    prop("sortable", "bool").default("false").doc("Turns the header into a sort button."),
                    prop("render", "fn(&T) -> Element").default("None").doc("Replaces the cell body. Sorting still uses `value`."),
                    prop("align", "CellAlign").default("follows the cell type").doc("Overrides the alignment the cell type chose."),
                    prop("row_header", "bool").default("false").doc("Renders the column's cells as `th scope=\"row\"`, so a screen reader names each row by it. One per table, usually the first."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Tab"], "With `scroll: true`: enters the scroll region, a tab stop.")
                .key(["Left", "Right", "Up", "Down"], "In the scroll region: scrolls the table.")
                .key(["Enter", "Space"], "On a sortable header, a button: sorts by that column.")
                .handles([
                    "An unnamed table warns in a debug build.",
                    "With `scroll: true` the wrapper is a `role=\"region\"` named like the table.",
                    "Only the sorted header carries `aria-sort`.",
                    "`.row_header()` cells render as `th scope=\"row\"`, so a screen reader reads that name as it moves down any other column. They look like the other cells.",
                ])
                .must([
                    "Name every table. `caption` shows a title and names it, `aria_labelledby` points at a heading already on the page, and `aria_label` names it without text.",
                    "Set `scroll: true` on a table wider than its container.",
                    "Mark the column that names a row with `.row_header()`.",
                ]),
            lead: rsx! {
                Text {
                    "A table built from "
                    Code { source: "data" }
                    " and "
                    Code { source: "columns" }
                    ". Each column comes from "
                    Code { source: "column" }
                    ", with a header and a "
                    Code { source: "value" }
                    " that reads one cell out of a row. The cell's type sets the sort order "
                    "and alignment, so a numeric column sorts numerically and aligns right "
                    "on its own."
                }
                Text {
                    Code { source: "sortable" }
                    " turns a header into a button. The first click sorts ascending and the "
                    "next flips it. "
                    Code { source: "render" }
                    " changes only what a cell draws, so the Role column below still sorts "
                    "by its text."
                }
            },
            Demo {
                component: "Table",
                children_text: "",
                fixed: vec![
                    r#"caption: "Team members""#.to_string(),
                    "data: people".to_string(),
                    COLUMNS.to_string(),
                ],
                controls: vec![
                    Control::switch("scroll"),
                    Control::switch("empty").code(|_, values| match no_rows(values) {
                        true => vec![r#"empty: rsx! { "No team members yet." }"#.to_string()],
                        false => vec![],
                    }),
                ],
                wrap: Wrap(wrap_data),
                render: move |values: DemoValues| rsx! {
                    Table {
                        caption: "Team members",
                        scroll: values.str("scroll") == "true",
                        empty: no_rows(&values).then(|| rsx! { "No team members yet." }),
                        data: if no_rows(&values) { Vec::new() } else { people() },
                        columns: vec![
                            column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
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
