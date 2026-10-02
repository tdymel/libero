//! Windowed and infinite tables: which rows render and how they are counted.

use crate::common::{attributes_of, body, render, tag_with, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Table, column},
};

/// 1156-5a: a windowed body renders a screenful, each row placed in the whole count.
#[test]
fn a_windowed_table_renders_a_window_and_counts_every_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Numbers",
                    max_height: "300px",
                    virtual_row_height: 40.0,
                    data: (0..1000u32).collect::<Vec<_>>(),
                    columns: vec![column("N").value(|n: &u32| *n)],
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(attributes_of(&html, "table")["aria-rowcount"], "1001");
    // The server has no viewport: a 1080px window, never all 1000 rows.
    let rows: Vec<String> = tags_with(&body, "<tr")
        .iter()
        .map(|row| row["aria-rowindex"].clone())
        .collect();
    // The header and the 32 rows of the server's window, never all 1000.
    let expected: Vec<String> = (1..=33).map(|index: u32| index.to_string()).collect();
    assert_eq!(rows, expected, "the header, then one window from the top");
}

/// 1156-5d: the live region that says appended rows is there before the first batch.
#[test]
fn an_infinite_table_renders_its_live_region_up_front() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Numbers",
                    max_height: "300px",
                    onbottomreached: |_| {},
                    data: (0..50u32).collect::<Vec<_>>(),
                    columns: vec![column("N").value(|n: &u32| *n)],
                }
            }
        }
    }

    let html = render(app);
    assert!(html.contains("role=\"status\""), "{html}");
}

/// 1156-5a: a detail row breaks the one row height, so `row_detail` renders every row.
#[test]
fn a_windowed_table_with_row_details_renders_every_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Numbers",
                    max_height: "300px",
                    virtual_row_height: 40.0,
                    data: (0..200u32).collect::<Vec<_>>(),
                    columns: vec![column("N").value(|n: &u32| *n)],
                    row_key: |n: &u32| n.to_string(),
                    row_detail: |n: &u32| Some(rsx! { "Detail {n}" }),
                }
            }
        }
    }

    let html = render(app);

    assert!(!attributes_of(&html, "table").contains_key("aria-rowcount"));
    // The header row, then all 200.
    assert_eq!(body(&html).matches("<tr").count(), 201);
}

/// 1156-5a with 3b: the header filters row counts among the windowed rows.
#[test]
fn a_windowed_table_counts_its_header_filters_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Numbers",
                    max_height: "300px",
                    virtual_row_height: 40.0,
                    header_filters: true,
                    data: (0..1000u32).collect::<Vec<_>>(),
                    columns: vec![column("N").value(|n: &u32| *n)],
                }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "table")["aria-rowcount"], "1002");
    assert_eq!(tag_with(&html, "data-filters=true")["aria-rowindex"], "2");
    assert!(body(&html).contains("aria-rowindex=\"3\""));
}

/// 1156-5c: an empty windowed table still counts the one row it shows.
#[test]
fn an_empty_windowed_table_counts_its_empty_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Numbers",
                    max_height: "300px",
                    virtual_row_height: 40.0,
                    data: Vec::<u32>::new(),
                    columns: vec![column("N").value(|n: &u32| *n)],
                }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "table")["aria-rowcount"], "2");
    assert!(body(&html).contains("aria-rowindex=\"2\""), "{html}");
}
