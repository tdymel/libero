//! The footer row: each aggregate over the filtered rows of every page, its
//! cells aligned and pinned like the body's, and its lead cell.

use crate::common::{body, render, tags_with};
use crate::table_fixture::{Stock, cell_of, table_rule, table_state};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Aggregate, PinnedColumns, Table, column},
};

fn stock() -> Vec<Stock> {
    vec![
        Stock {
            id: 1,
            name: "Ada",
            cents: 100,
        },
        Stock {
            id: 2,
            name: "Alan",
            cents: 250,
        },
        Stock {
            id: 3,
            name: "Grace",
            cents: 400,
        },
    ]
}

fn foot(html: &str) -> String {
    let body = body(html);
    let start = body
        .find("<tfoot")
        .unwrap_or_else(|| panic!("no <tfoot> in {body}"));
    body[start..start + body[start..].find("</tfoot>").unwrap()].to_string()
}

#[test]
fn a_table_without_an_aggregate_has_no_footer() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    columns: vec![column("Cents").value(|row: &Stock| row.cents)],
                }
            }
        }
    }

    assert!(!body(&render(app)).contains("<tfoot"));
}

#[test]
fn every_aggregate_reads_the_numbers_and_names_itself() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    columns: vec![
                        column("Name").value(|row: &Stock| row.name.to_string()).aggregate(Aggregate::Count),
                        column("Id").value(|row: &Stock| row.id).aggregate(Aggregate::Max),
                        column("Cents").value(|row: &Stock| row.cents).aggregate(Aggregate::Sum),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let footer = foot(&html);
    assert!(
        footer.contains(">Count<") && footer.contains(">3<"),
        "{footer}"
    );
    assert!(
        footer.contains(">Maximum<") && footer.contains(">3<"),
        "{footer}"
    );
    assert!(
        footer.contains(">Sum<") && footer.contains(">750<"),
        "{footer}"
    );
    // After the rows, aligned like its column.
    assert!(body(&html).find("</tbody>").unwrap() < body(&html).find("<tfoot").unwrap());
    let sum = cell_of(&footer, "td", "750");
    assert_eq!(sum["data-align"], "end", "{sum:?}");
}

#[test]
fn the_footer_covers_the_filtered_rows_of_every_page() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    default_page_size: 1usize,
                    default_quick_filter: "a",
                    columns: vec![
                        column("Name").value(|row: &Stock| row.name.to_string()),
                        column("Cents")
                            .value(|row: &Stock| row.cents)
                            .aggregate(Aggregate::Sum)
                            .aggregate_format(|cents| format!("{:.2} EUR", cents / 100.0)),
                    ],
                }
            }
        }
    }

    // Ada, Alan and Grace all contain an "a"; one row shows, all three add up.
    let footer = foot(&render(app));
    assert!(footer.contains(">7.50 EUR<"), "{footer}");
}

#[test]
fn the_quick_filter_narrows_the_footer() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    default_quick_filter: "gra",
                    columns: vec![
                        column("Name").value(|row: &Stock| row.name.to_string()),
                        column("Cents").value(|row: &Stock| row.cents).aggregate(Aggregate::Avg),
                    ],
                }
            }
        }
    }

    let footer = foot(&render(app));
    assert!(
        footer.contains(">Average<") && footer.contains(">400<"),
        "{footer}"
    );
}

#[test]
fn a_manual_pagination_footer_covers_the_loaded_rows() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    manual_pagination: true,
                    default_page_size: 3usize,
                    row_count: 90usize,
                    columns: vec![column("Cents").value(|row: &Stock| row.cents).aggregate(Aggregate::Sum)],
                }
            }
        }
    }

    assert!(foot(&render(app)).contains(">750<"));
}

#[test]
fn the_lead_columns_are_spanned_by_one_cell() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    selectable: true,
                    row_detail: |_: &Stock| Some(rsx! { "More" }),
                    columns: vec![column("Cents").value(|row: &Stock| row.cents).aggregate(Aggregate::Sum)],
                }
            }
        }
    }

    let html = render(app);
    let lead = tags_with(&foot(&html), "data-footer-lead");
    assert_eq!(lead.len(), 1, "{lead:?}");
    assert_eq!(lead[0]["colspan"], "2", "{lead:?}");
}

#[test]
fn a_pinned_column_pins_its_footer_cell_and_a_pinned_lead_holds_the_lead_cell() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    selectable: true,
                    scroll: true,
                    default_pinned_columns: PinnedColumns::default().start(["Cents"]),
                    columns: vec![
                        column("Name").value(|row: &Stock| row.name.to_string()),
                        column("Cents").value(|row: &Stock| row.cents).aggregate(Aggregate::Sum),
                    ],
                }
            }
        }
    }

    let html = render(app);
    let cents = cell_of(&foot(&html), "td", "750");
    assert_eq!(cents["data-pin"], "start", "{cents:?}");
    assert!(
        table_state(&html).contains(&"pin-footer-lead".to_string()),
        "{html}"
    );
    let lead = table_rule(
        &html,
        r#"[data-state~="pin-footer-lead"] tfoot td[data-footer-lead]"#,
    );
    assert_eq!(lead["position"], "sticky", "{lead:?}");
}

#[test]
fn a_capped_table_sticks_its_footer_at_the_bottom() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: stock(),
                    max_height: "10rem",
                    columns: vec![column("Cents").value(|row: &Stock| row.cents).aggregate(Aggregate::Sum)],
                }
            }
        }
    }

    let html = render(app);
    assert!(
        table_state(&html).contains(&"sticky-footer".to_string()),
        "{html}"
    );
    let rule = table_rule(&html, r#"[data-state~="sticky-footer"] tfoot td"#);
    assert_eq!(rule["position"], "sticky", "{rule:?}");
    assert_eq!(rule["bottom"], "0", "{rule:?}");
}

#[test]
fn the_footer_waits_while_the_first_rows_load() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    aria_label: "Stock",
                    data: Vec::<Stock>::new(),
                    loading: true,
                    columns: vec![column("Cents").value(|row: &Stock| row.cents).aggregate(Aggregate::Sum)],
                }
            }
        }
    }

    assert!(!body(&render(app)).contains("<tfoot"));
}
