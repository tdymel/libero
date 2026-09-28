use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Column, SortDirection, Table, TableSort, column},
};

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
}

fn people() -> Vec<Person> {
    vec![
        Person {
            name: "Ada",
            age: 36,
        },
        Person {
            name: "Grace",
            age: 45,
        },
        Person {
            name: "Linus",
            age: 28,
        },
    ]
}

fn columns() -> Vec<Column<Person>> {
    vec![
        column("Name")
            .value(|row: &Person| row.name.to_string())
            .sortable()
            .row_header(),
        column("Age").value(|row: &Person| row.age).sortable(),
        column("Initial").value(|row: &Person| row.name[..1].to_string()),
    ]
}

/// The sort names its column by header text, so it follows the column (4c).
#[test]
fn a_column_order_moves_headers_and_cells_and_the_sort_follows() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(),
                    default_column_order: vec!["Age".to_string(), "Initial".to_string()],
                    default_sort: vec![TableSort::new("Name", SortDirection::Descending)],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let at = |text: &str| body.find(text).unwrap_or_else(|| panic!("{text}: {body}"));

    assert!(at("Age<") < at(">Initial<") && at(">Initial<") < at("Name<"));
    // The sorted header is still Name's.
    let sorted = at("aria-sort=\"descending\"");
    assert!(sorted > at(">Initial<") && sorted < at("Name<"));
    // Cells follow their header; Linus leads, sorted by name descending.
    assert!(at(">28<") < at(">L<") && at(">L<") < at(">Linus<"));
    assert!(at(">Linus<") < at(">Grace<"));
}

#[test]
fn row_reorder_adds_a_handle_column_with_move_buttons() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(),
                    row_key: |row: &Person| row.name.to_string(),
                    onrowreorder: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains(">Reorder<"), "{body}");
    assert_eq!(body.matches("data-reorder-handle").count(), 3);
    assert_eq!(body.matches("data-reorder-move=\"up\"").count(), 3);
    assert!(body.contains("Move Ada up"), "{body}");
    assert!(!body.contains("transform"), "{body}");
    // The first row has nothing above. "Down" waits for the rows to mount and count.
    assert!(
        body.contains("aria-label=\"Move Ada up\" disabled"),
        "{body}"
    );
    assert!(
        !body.contains("aria-label=\"Move Grace up\" disabled"),
        "{body}"
    );
}

/// The shown order is a view over `data` while sorted: nothing moves (4b).
#[test]
fn row_reorder_is_off_while_sorted() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: people(),
                    columns: columns(),
                    row_key: |row: &Person| row.name.to_string(),
                    default_sort: vec![TableSort::new("Age", SortDirection::Ascending)],
                    onrowreorder: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(body.matches(" disabled").count(), 9, "{body}");
}
