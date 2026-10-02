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

#[test]
fn a_row_box_renders_as_a_plain_checkbox() {
    fn table() -> Element {
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
    // The table's lighter box (todo 1195) must stay the same markup, ids aside.
    let table = body(&render(table));
    let cell = table.split("<td data-select=true>").nth(1).unwrap();
    let cell = &cell[..cell.find("</td>").unwrap()];
    let plain = render(checkbox);
    // The provider's empty portal follows the box.
    let plain = body(&plain)
        .strip_suffix("<div></div>")
        .unwrap()
        .to_string();
    let id = plain.find(" id=\"").unwrap();
    let end = id + 5 + plain[id + 5..].find('"').unwrap() + 1;
    assert_eq!(cell, format!("{}{}", &plain[..id], &plain[end..]));
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
