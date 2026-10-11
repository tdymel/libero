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

/// The first header cell until focus moves it; the leading checkbox cell is no grid cell.
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
    let stops = tags_with(&html, "tabindex=\"0\"");
    assert_eq!(stops.len(), 1, "{html}");
    assert!(stops[0].contains_key("data-cell-stop"), "{html}");
    assert_eq!(stops[0]["scope"], "col", "the first column header");
    assert_eq!(tags_with(body, "tabindex=\"-1\"").len(), 4, "{html}");
    let table = attributes_of(&html, "table");
    assert_eq!(table["aria-multiselectable"], "true");
    assert_eq!(table["aria-readonly"], "true");
    // The checkbox cells carry no tabindex of their own.
    assert!(
        tags_with(body, "<td")
            .iter()
            .filter(|cell| cell.contains_key("data-select"))
            .all(|cell| !cell.contains_key("tabindex")),
        "{html}"
    );
}

/// A windowed grid renders one stop as well, in its header.
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
    assert_eq!(tags_with(&html, "tabindex=\"0\"").len(), 1, "{html}");
    assert!(
        tags_with(tbody(&html), "tabindex=\"0\"").is_empty(),
        "{html}"
    );
}

/// Group rows stay out of the grid: no cell of theirs is a stop.
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
    let stops = tags_with(&html, "tabindex=\"0\"");
    assert_eq!(stops.len(), 1, "{html}");
    assert_eq!(stops[0]["scope"], "col", "{html}");
    assert!(
        tags_with(tbody(&html), "<td")
            .iter()
            .filter(|cell| !cell.contains_key("tabindex"))
            .count()
            > 0,
        "the group rows' cells are no grid cells: {html}"
    );
}

/// The header cells over a group are grid cells too, the group's first.
#[test]
fn group_header_cells_are_grid_cells() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    cell_focus: true,
                    data: people(),
                    columns: vec![
                        column("Name").value(|p: &Person| p.name).group("Person"),
                        column("Age").value(|p: &Person| p.age).group("Person"),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let head = &html[html.find("<thead").expect("a thead")..html.find("</thead>").expect("closed")];
    let stops = tags_with(head, "tabindex=\"0\"");
    assert_eq!(stops.len(), 1, "{html}");
    assert_eq!(stops[0]["scope"], "colgroup", "{html}");
    assert_eq!(tags_with(head, "tabindex=\"-1\"").len(), 2, "{html}");
}

/// A paged grid counts every page's rows: the header row, then two of three.
#[test]
fn the_grid_counts_rows_when_paged() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    cell_focus: true,
                    default_page_size: 2,
                    data: vec![
                        Person { name: "Ana", age: 31 },
                        Person { name: "Ben", age: 42 },
                        Person { name: "Cy", age: 50 },
                    ],
                    columns: columns(),
                }
            }
        }
    }

    let html = render(app);
    assert_eq!(
        attributes_of(&html, "table")["aria-rowcount"],
        "4",
        "{html}"
    );
    let rows = tags_with(&html, "<tr");
    let indices: Vec<&str> = rows
        .iter()
        .map(|row| row["aria-rowindex"].as_str())
        .collect();
    assert_eq!(indices, ["1", "2", "3"], "{html}");
}

/// A hidden column keeps its place in the count: Name 1, City 3.
#[test]
fn a_hidden_column_leaves_a_colindex_gap() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    cell_focus: true,
                    default_hidden_columns: vec!["Age".to_string()],
                    data: people(),
                    columns: vec![
                        column("Name").value(|p: &Person| p.name),
                        column("Age").value(|p: &Person| p.age),
                        column("City").value(|_: &Person| "Rome"),
                    ],
                }
            }
        }
    }

    let html = render(app);
    assert_eq!(
        attributes_of(&html, "table")["aria-colcount"],
        "3",
        "{html}"
    );
    let cells = tags_with(&html, "aria-colindex=");
    let indices: Vec<&str> = cells
        .iter()
        .map(|cell| cell["aria-colindex"].as_str())
        .collect();
    assert_eq!(indices, ["1", "3", "1", "3", "1", "3"], "{html}");
}

/// Without a hidden column no cell names its place.
#[test]
fn a_grid_without_hidden_columns_has_no_colindex() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table { caption: "People", cell_focus: true, data: people(), columns: columns() }
            }
        }
    }

    let html = render(app);
    assert!(!html.contains("aria-colindex"), "{html}");
    assert!(!html.contains("aria-rowindex"), "{html}");
}

fn editable_columns() -> Vec<libero::components::Column<Person>> {
    vec![
        column("Name").value(|p: &Person| p.name).row_header(),
        column("Age")
            .value(|p: &Person| p.age)
            .editable(|p: &mut Person, age: u32| p.age = age),
    ]
}

#[test]
fn an_editable_column_makes_a_grid() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: editable_columns(),
                    onrowupdate: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let table = attributes_of(&html, "table");
    assert_eq!(table["role"], "grid");
    assert!(!table.contains_key("aria-readonly"), "{html}");
}

#[test]
fn a_non_editable_cell_is_readonly_in_an_editable_grid() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: editable_columns(),
                    onrowupdate: |_| {},
                }
            }
        }
    }

    let html = render(app);
    let cells = tags_with(tbody(&html), "tabindex=");
    assert_eq!(cells.len(), 4, "{html}");
    // The row headers (Name) are read-only, the Age cells are not.
    let readonly: Vec<bool> = cells
        .iter()
        .map(|cell| cell.get("aria-readonly").map(String::as_str) == Some("true"))
        .collect();
    assert_eq!(readonly, [true, false, true, false], "{html}");
    assert!(!html.contains("data-editing"), "{html}");
}
