//! Row selection: the checkbox column, select-all and its mixed state.

use crate::common::{body, render};
use crate::table_fixture::{Stock, people, person_columns};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Checkbox, Table, column},
    theme::Size,
};

fn selectable(selection: Vec<String>) -> Element {
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "Stock",
                selectable: true,
                default_selection: selection,
                data: vec![
                    Stock { id: 7, name: "Apple", cents: 120 },
                    Stock { id: 9, name: "Pear", cents: 5 },
                ],
                columns: vec![
                    column("Name").value(|s: &Stock| s.name.to_string()).row_header(),
                    column("Price").value(|s: &Stock| s.cents),
                ],
                row_key: |s: &Stock| s.id.to_string(),
            }
        }
    }
}

#[test]
fn a_selectable_table_marks_selected_rows_and_mixes_select_all() {
    let body = body(&render(|| selectable(vec!["9".into()])));

    assert_eq!(
        body.matches("aria-label=\"Select all rows\"").count(),
        1,
        "{body}"
    );
    assert!(body.contains("aria-label=\"Select Apple\""), "{body}");
    assert!(body.contains("aria-label=\"Select Pear\""), "{body}");
    assert_eq!(body.matches("aria-selected=\"true\"").count(), 1, "{body}");
    assert_eq!(body.matches("aria-selected=\"false\"").count(), 1, "{body}");
    assert_eq!(body.matches("aria-checked=\"mixed\"").count(), 1, "{body}");
    assert_eq!(body.matches("role=\"status\"").count(), 1, "{body}");
    assert_eq!(body.matches("<th scope=\"col\"").count(), 3, "{body}");
}

#[test]
fn select_all_is_checked_only_with_every_row_selected() {
    let all = body(&render(|| {
        selectable(vec!["7".into(), "9".into(), "x".into()])
    }));
    assert!(!all.contains("aria-checked=\"mixed\""), "{all}");
    assert_eq!(all.matches("aria-selected=\"true\"").count(), 2, "{all}");

    let none = body(&render(|| selectable(Vec::new())));
    assert!(!none.contains("aria-checked=\"mixed\""), "{none}");
    assert!(!none.contains("aria-selected=\"true\""), "{none}");
}

fn sm_table() -> Element {
    rsx! {
        LiberoProvider {
            Table {
                aria_label: "Stock",
                size: Size::Sm,
                selectable: true,
                default_selection: vec!["7".to_string()],
                data: vec![Stock { id: 7, name: "Apple", cents: 120 }],
                columns: vec![column("Name").value(|s: &Stock| s.name.to_string()).row_header()],
                row_key: |s: &Stock| s.id.to_string(),
            }
        }
    }
}

/// The markup of the first body row's checkbox cell, inside its `<td>`.
fn row_cell(table: &str) -> String {
    let cell = table.split("<td data-select=true>").nth(1).unwrap();
    cell[..cell.find("</td>").unwrap()].to_string()
}

/// The first `<input ...>` tag of `html`, its `id` dropped.
fn input_tag(html: &str) -> String {
    let start = html.find("<input").unwrap();
    let tag = &html[start..start + html[start..].find('>').unwrap() + 1];
    match tag.find(" id=\"") {
        Some(id) => {
            let end = id + 5 + tag[id + 5..].find('"').unwrap() + 1;
            format!("{}{}", &tag[..id], &tag[end..])
        }
        None => tag.to_string(),
    }
}

/// The table's lighter box (todo 1982) keeps the plain Checkbox's input: role, name and state.
#[test]
fn a_row_box_has_the_input_of_a_plain_checkbox() {
    fn checkbox() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox {
                    aria_label: "Select Apple",
                    size: Size::Sm,
                    checked: true,
                    onchange: |_| {},
                }
            }
        }
    }
    let cell = row_cell(&body(&render(sm_table)));
    let plain = body(&render(checkbox));
    assert_eq!(input_tag(&cell), input_tag(&plain), "{cell}");
}

/// A row's checkbox cell holds the control, the input, the box and its mark: 5 elements (todo 1982).
#[test]
fn a_row_box_stays_within_its_element_budget() {
    let cell = row_cell(&body(&render(sm_table)));
    let elements = cell.matches('<').count() - cell.matches("</").count();
    assert!(elements <= 5, "{elements} elements: {cell}");
}

#[test]
fn a_table_without_selectable_has_no_checkbox_column() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table { aria_label: "People", data: people(), columns: person_columns() }
            }
        }
    }

    let body = body(&render(app));

    assert!(!body.contains("aria-selected"), "{body}");
    assert!(!body.contains("data-select"), "{body}");
    assert!(!body.contains("role=\"status\""), "{body}");
}
