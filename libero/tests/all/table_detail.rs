use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{RowFn, Table, column},
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

/// 1392: the predicate decides the toggles; only open rows build their detail.
#[test]
fn row_has_detail_builds_the_open_details_only() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static BUILT: AtomicUsize = AtomicUsize::new(0);

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Row| row.name.to_string()).row_header()],
                    row_key: |row: &Row| row.id.to_string(),
                    row_detail: |row: &Row| {
                        BUILT.fetch_add(1, Ordering::Relaxed);
                        detail(row)
                    },
                    row_has_detail: |row: &Row| row.note.is_some(),
                    default_expanded: vec!["3".to_string()],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(body.matches("data-detail-button").count(), 2, "{body}");
    assert!(body.contains("Likes tables"), "{body}");
    assert!(!body.contains("Wrote the first program"), "{body}");
    assert_eq!(BUILT.load(Ordering::Relaxed), 1);
}

/// 1391: an open row slides in its detail; closed rows render none.
#[test]
fn animate_details_wraps_the_open_detail_in_a_collapse() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: vec![column("Name").value(|row: &Row| row.name.to_string()).row_header()],
                    row_key: |row: &Row| row.id.to_string(),
                    row_detail: detail,
                    animate_details: true,
                    default_expanded: vec!["1".to_string()],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert_eq!(body.matches("data-sliding=true").count(), 1, "{body}");
    assert!(body.contains("data-detail-body"), "{body}");
    assert!(body.contains("Wrote the first program"), "{body}");
    assert!(!body.contains("Likes tables"), "{body}");
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

/// 1440: turning `row_detail` on re-renders the table; `RowFn` compared equal
/// to an unset one, so the toggles never showed.
#[test]
fn setting_row_detail_later_shows_the_toggles() {
    use std::cell::Cell;
    thread_local!(static ON: Cell<Option<Signal<bool>>> = const { Cell::new(None) });

    #[component]
    fn People(on: bool) -> Element {
        rsx! {
            Table {
                caption: "People",
                data: people(),
                columns: vec![column("Name").value(|row: &Row| row.name.to_string()).row_header()],
                row_key: |row: &Row| row.id.to_string(),
                row_detail: match on {
                    true => detail.into(),
                    false => RowFn::default(),
                },
            }
        }
    }

    fn app() -> Element {
        let on = use_signal(|| false);
        ON.set(Some(on));
        rsx! {
            LiberoProvider { People { on: on() } }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    assert!(!body(&dioxus_ssr::render(&dom)).contains("data-detail-button"));
    dom.in_runtime(|| ON.get().expect("the switch").set(true));
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = body(&dioxus_ssr::render(&dom));
    assert_eq!(html.matches("data-detail-button").count(), 2, "{html}");
}
