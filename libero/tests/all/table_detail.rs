use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{PinnedColumns, Table, column},
};

#[derive(Clone, PartialEq)]
struct Row {
    id: u32,
    name: &'static str,
    note: Option<&'static str>,
}

fn people() -> Vec<Row> {
    vec![
        Row {
            id: 1,
            name: "Ada",
            note: Some("Wrote the first program"),
        },
        Row {
            id: 2,
            name: "Bea",
            note: None,
        },
        Row {
            id: 3,
            name: "Cy",
            note: Some("Likes tables"),
        },
    ]
}

fn detail(row: &Row) -> Option<Element> {
    row.note.map(|note| rsx! { p { "{note}" } })
}

#[test]
fn rows_with_a_detail_get_a_closed_toggle_and_no_detail_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Row| row.name.to_string()).row_header()],
                    row_key: |row: &Row| row.id.to_string(),
                    row_detail: detail,
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    // A toggle column on every row and in the header; buttons only where a detail exists.
    assert_eq!(body.matches("data-detail-toggle").count(), 4, "{body}");
    assert_eq!(body.matches("data-detail-button").count(), 2, "{body}");
    assert_eq!(body.matches("aria-expanded=\"false\"").count(), 2);
    assert!(body.contains("aria-label=\"Details for Ada\""), "{body}");
    assert!(
        body.contains("Details</"),
        "the header names the column: {body}"
    );
    // Closed: no detail row, and nothing to point at.
    assert!(!body.contains("data-detail=true"), "{body}");
    assert!(!body.contains("aria-controls"));
    assert!(!body.contains("Wrote the first program"));
}

#[test]
fn an_open_detail_follows_its_row_across_every_column() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: vec![
                        column("Name").value(|row: &Row| row.name.to_string()).row_header(),
                        column("Id").value(|row: &Row| row.id),
                    ],
                    row_key: |row: &Row| row.id.to_string(),
                    row_detail: detail,
                    selectable: true,
                    default_expanded: vec!["1".to_string()],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let detail_at = body
        .find("data-detail=true")
        .unwrap_or_else(|| panic!("a detail row: {body}"));
    let ada = body.find(">Ada<").unwrap();
    let bea = body.find(">Bea<").unwrap();
    assert!(ada < detail_at && detail_at < bea, "{body}");
    // Toggle, checkbox and two data columns.
    assert!(body[detail_at..].contains("colspan=\"4\""), "{body}");
    assert!(body.contains("Wrote the first program"));
    assert!(!body.contains("Likes tables"));

    // The open toggle points at the detail row's id.
    let controls = body
        .split("aria-controls=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("aria-controls on the open toggle");
    assert!(body.contains(&format!("id=\"{controls}\"")), "{body}");
    assert_eq!(body.matches("aria-expanded=\"true\"").count(), 1);
}

#[test]
fn stripes_count_rows_not_their_details() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Row| row.name.to_string())],
                    row_detail: detail,
                    striped: true,
                    default_expanded: vec!["0".to_string()],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert!(body.contains("data-detail=true"), "{body}");
    // Only Bea, the second row, is striped, though the detail row sits before it.
    assert_eq!(body.matches("data-stripe").count(), 1, "{body}");
    let stripe = body.find("data-stripe").unwrap();
    assert!(body[stripe..].find(">Bea<").unwrap() < body[stripe..].find("</tr>").unwrap());
}

#[test]
fn a_start_pin_holds_the_toggles_and_insets_past_them_and_the_checkboxes() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    scroll: true,
                    data: people(),
                    columns: vec![column("Name").value(|row: &Row| row.name.to_string())],
                    row_key: |row: &Row| row.id.to_string(),
                    row_detail: detail,
                    selectable: true,
                    default_pinned_columns: PinnedColumns::default().start(["Name"]),
                }
            }
        }
    }

    let html = render(app);
    assert!(html.contains("pin-detail"), "{html}");
    let body = body(&html);
    let ada = body.find(">Ada<").unwrap();
    let cell = &body[body[..ada].rfind("<td").unwrap()..ada];
    // The toggle column's 24px, then the checkbox column's.
    assert!(cell.contains("inset-inline-start:calc(calc(24px"), "{cell}");
}
