//! `cell_focus`: the grid role and its one tab stop among the body cells.

use crate::common::{attributes_of, render, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Table, column},
};

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
}

fn people() -> Vec<Person> {
    vec![
        Person {
            name: "Ana",
            age: 31,
        },
        Person {
            name: "Ben",
            age: 42,
        },
    ]
}

fn columns() -> Vec<libero::components::Column<Person>> {
    vec![
        column("Name").value(|p: &Person| p.name).row_header(),
        column("Age").value(|p: &Person| p.age),
    ]
}

/// The body, where the grid's cells are.
fn tbody(html: &str) -> &str {
    let start = html.find("<tbody").expect("a tbody");
    &html[start..html.find("</tbody>").expect("a closed tbody")]
}

#[test]
fn a_cell_focus_table_is_a_grid() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table { caption: "People", cell_focus: true, data: people(), columns: columns() }
            }
        }
    }

    let html = render(app);
    assert_eq!(attributes_of(&html, "table")["role"], "grid");
    let cells = tags_with(tbody(&html), "tabindex=");
    // Two rows of a row header and a cell; the web reads a `td` as a gridcell.
    assert_eq!(cells.len(), 4, "{html}");
    assert!(
        cells.iter().all(|cell| !cell.contains_key("role")),
        "{html}"
    );
}

#[test]
fn a_plain_table_has_no_grid_markup() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table { caption: "People", data: people(), columns: columns() }
            }
        }
    }

    let html = render(app);
    assert!(
        !attributes_of(&html, "table").contains_key("role"),
        "{html}"
    );
    assert!(tags_with(tbody(&html), "tabindex=").is_empty(), "{html}");
    assert!(!html.contains("data-cell-stop"), "{html}");
}

/// The first body cell until focus moves it; the leading checkbox cell is no grid cell.
#[test]
fn exactly_one_cell_is_a_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    cell_focus: true,
                    selectable: true,
                    row_key: |p: &Person| p.name.to_string(),
                    data: people(),
                    columns: columns(),
                }
            }
        }
    }

    let html = render(app);
    let body = tbody(&html);
    let stops = tags_with(body, "tabindex=\"0\"");
    assert_eq!(stops.len(), 1, "{html}");
    assert!(stops[0].contains_key("data-cell-stop"), "{html}");
    assert_eq!(stops[0]["scope"], "row", "the row header of the first row");
    assert_eq!(tags_with(body, "tabindex=\"-1\"").len(), 3, "{html}");
    // The checkbox cells carry no tabindex of their own.
    assert!(
        tags_with(body, "<td")
            .iter()
            .filter(|cell| cell.contains_key("data-select"))
            .all(|cell| !cell.contains_key("tabindex")),
        "{html}"
    );
}

/// A windowed grid renders one stop as well, in its first row.
#[test]
fn a_windowed_grid_has_one_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Numbers",
                    cell_focus: true,
                    max_height: "300px",
                    virtual_row_height: 40.0,
                    data: (0..1000u32).collect::<Vec<_>>(),
                    columns: vec![column("N").value(|n: &u32| *n), column("Double").value(|n: &u32| n * 2)],
                }
            }
        }
    }

    let html = render(app);
    assert_eq!(attributes_of(&html, "table")["role"], "grid");
    assert_eq!(tags_with(tbody(&html), "tabindex=\"0\"").len(), 1, "{html}");
}

/// Group rows stay out of the grid: the one stop is in a data row, under its group row.
#[test]
fn a_grouped_grid_has_one_stop_in_a_data_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    cell_focus: true,
                    default_row_grouping: vec!["Age".to_string()],
                    default_group_expansion: 1,
                    row_key: |p: &Person| p.name.to_string(),
                    data: people(),
                    columns: columns(),
                }
            }
        }
    }

    let html = render(app);
    let stops = tags_with(tbody(&html), "tabindex=\"0\"");
    assert_eq!(stops.len(), 1, "{html}");
    assert!(stops[0].contains_key("data-cell-stop"), "{html}");
    assert_eq!(stops[0]["scope"], "row", "{html}");
}
