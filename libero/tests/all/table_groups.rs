use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{PinnedColumns, Table, column},
};

#[derive(Clone, PartialEq)]
struct Row {
    first: &'static str,
    last: &'static str,
    age: u32,
}

fn rows() -> Vec<Row> {
    vec![Row {
        first: "Ada",
        last: "Lovelace",
        age: 36,
    }]
}

#[test]
fn a_group_heads_its_adjacent_columns_and_the_others_span_down() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    selectable: true,
                    row_key: |row: &Row| row.first.to_string(),
                    data: rows(),
                    columns: vec![
                        column("First").value(|row: &Row| row.first).group("Name"),
                        column("Last").value(|row: &Row| row.last).group("Name"),
                        column("Age").value(|row: &Row| row.age),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let head = &body[body.find("<thead").unwrap()..body.find("</thead>").unwrap()];

    assert_eq!(head.matches("<tr").count(), 2, "{head}");
    assert!(
        head.contains("<th scope=\"colgroup\" colspan=\"2\" data-group=true>Name</th>"),
        "{head}"
    );
    // The select-all box and the ungrouped column fill both header rows.
    assert_eq!(head.matches("rowspan=\"2\"").count(), 2, "{head}");
    assert!(
        head.contains("<th scope=\"col\" rowspan=\"2\" data-select"),
        "{head}"
    );
    assert_eq!(head.matches("<th scope=\"col\"").count(), 4, "{head}");
}

#[test]
fn nested_groups_merge_under_the_same_parent_only() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    data: rows(),
                    columns: vec![
                        column("First").value(|row: &Row| row.first).group("A").group("X"),
                        column("Last").value(|row: &Row| row.last).group("A").group("X"),
                        column("Age").value(|row: &Row| row.age).group("B").group("X"),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let head = body(&html);
    let head = &head[head.find("<thead").unwrap()..head.find("</thead>").unwrap()];

    assert_eq!(head.matches("<tr").count(), 3, "{head}");
    assert_eq!(head.matches(">X</th>").count(), 2, "{head}");
    assert!(
        head.contains("colspan=\"2\" data-group=true>X</th>"),
        "{head}"
    );
    assert!(
        head.contains("<th scope=\"colgroup\" data-group=true>B</th>"),
        "{head}"
    );
}

#[test]
fn a_hidden_column_narrows_its_group_and_an_emptied_group_goes() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    default_hidden_columns: vec!["Last".to_string(), "Age".to_string()],
                    data: rows(),
                    columns: vec![
                        column("First").value(|row: &Row| row.first).group("Name"),
                        column("Last").value(|row: &Row| row.last).group("Name"),
                        column("Age").value(|row: &Row| row.age).group("Facts"),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(
        body.contains("<th scope=\"colgroup\" data-group=true>Name</th>"),
        "{body}"
    );
    assert!(!body.contains("Facts"), "{body}");
}

#[test]
fn a_spanning_cell_covers_the_next_shown_columns_only() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    default_hidden_columns: vec!["Last".to_string()],
                    data: vec![
                        Row { first: "Ada", last: "Lovelace", age: 36 },
                        Row { first: "Total", last: "", age: 0 },
                    ],
                    columns: vec![
                        column("First")
                            .value(|row: &Row| row.first)
                            .col_span(|row: &Row| if row.first == "Total" { 2 } else { 1 }),
                        column("Last").value(|row: &Row| row.last),
                        column("Age").value(|row: &Row| row.age),
                        column("Note").value(|_: &Row| "n"),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // "Last" is hidden, so the span covers First and Age, and Note stays.
    assert!(
        body.contains("<td colspan=\"2\">Total</td><td>n</td>"),
        "{body}"
    );
    assert_eq!(body.matches("<td").count(), 3 + 2, "{body}");
}

#[test]
fn a_bounded_table_with_groups_sticks_its_thead() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    max_height: "10rem",
                    data: rows(),
                    columns: vec![
                        column("First").value(|row: &Row| row.first).group("Name"),
                        column("Age").value(|row: &Row| row.age),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let state = &attributes_of(&html, "table")["data-state"];

    assert!(state.contains("sticky-head"), "{state}");
    assert!(!state.contains("sticky-header"), "{state}");
}

#[test]
fn a_pin_edge_splits_a_group_and_stops_a_span() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "People",
                    default_pinned_columns: PinnedColumns::default().start(["First"]),
                    data: rows(),
                    columns: vec![
                        column("First")
                            .value(|row: &Row| row.first)
                            .group("Name")
                            .width("6rem")
                            .col_span(|_| 2),
                        column("Last").value(|row: &Row| row.last).group("Name"),
                        column("Age").value(|row: &Row| row.age),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let head = &body[body.find("<thead").unwrap()..body.find("</thead>").unwrap()];

    // The pinned half of the group sticks with its column (todo 1359), the other scrolls.
    assert!(
        head.contains(
            "<th scope=\"colgroup\" data-group=true data-pin=\"start\" data-pin-edge=true \
             style=\"inset-inline-start:0;\">Name</th>"
        ),
        "{head}"
    );
    assert!(
        head.contains("<th scope=\"colgroup\" data-group=true>Name</th>"),
        "{head}"
    );
    assert!(!head.contains("colgroup\" colspan"), "{head}");
    assert_eq!(head.matches("data-pin=\"start\"").count(), 2, "{head}");
    let rows = &body[body.find("<tbody").unwrap()..];
    assert!(
        rows.contains("<td data-pin=\"start\" data-pin-edge=true style=\"inset-inline-start:0;\">Ada</td><td>Lovelace</td>"),
        "{rows}"
    );
}
