use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Aggregate, Column, SortDirection, Table, TableSort, column},
};

#[derive(Clone, PartialEq)]
struct Row {
    name: &'static str,
    team: &'static str,
    city: &'static str,
}

fn people() -> Vec<Row> {
    [
        ("Ada", "Sales", "Berlin"),
        ("Bea", "Ops", "Rome"),
        ("Cy", "Sales", "Rome"),
        ("Dan", "", "Berlin"),
    ]
    .map(|(name, team, city)| Row { name, team, city })
    .to_vec()
}

fn columns() -> Vec<Column<Row>> {
    vec![
        column("Name")
            .value(|row: &Row| row.name.to_string())
            .sortable()
            .row_header(),
        column("Team").value(|row: &Row| row.team.to_string()),
        column("City").value(|row: &Row| row.city.to_string()),
    ]
}

fn team() -> Vec<String> {
    vec!["Team".to_string()]
}

/// Where `text` starts as a cell's or label's whole text.
fn at(body: &str, text: &str) -> usize {
    body.find(&format!(">{text}<"))
        .unwrap_or_else(|| panic!("no {text}: {body}"))
}

fn shows(body: &str, text: &str) -> bool {
    body.contains(&format!(">{text}<"))
}

#[test]
fn grouped_rows_start_collapsed_under_one_toggle_per_group() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table { caption: "People", data: people(), columns: columns(), default_row_grouping: team() }
            }
        }
    }

    let body = body(&render(app));
    assert_eq!(body.matches("data-group-toggle").count(), 3, "{body}");
    assert_eq!(body.matches("aria-expanded=\"false\"").count(), 3, "{body}");
    assert!(body.contains("aria-label=\"Sales, 2 rows\""), "{body}");
    assert!(body.contains("aria-label=\"Ops, 1 row\""), "{body}");
    // Empty cells form one group under the empty label, after the others.
    assert!(body.contains("aria-label=\"(empty), 1 row\""), "{body}");
    assert!(at(&body, "Sales") < at(&body, "Ops") && at(&body, "Ops") < at(&body, "(empty)"));
    assert!(
        body.contains("data-level=\"0\"") || body.contains("data-level=0"),
        "{body}"
    );
    for name in ["Ada", "Bea", "Cy", "Dan"] {
        assert!(!shows(&body, name), "{name} is in a closed group: {body}");
    }
    // The table keeps its role: the toggles are buttons, no tree semantics.
    assert!(
        !body.contains("treegrid") && !body.contains("aria-level"),
        "{body}"
    );
}

#[test]
fn an_open_group_shows_its_rows_under_its_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: columns(),
                    default_row_grouping: team(),
                    default_expanded_groups: vec!["Sales".to_string()],
                }
            }
        }
    }

    let body = body(&render(app));
    assert_eq!(body.matches("aria-expanded=\"true\"").count(), 1, "{body}");
    let (sales, ops) = (at(&body, "Sales"), at(&body, "Ops"));
    assert!(sales < at(&body, "Ada") && at(&body, "Ada") < at(&body, "Cy"));
    assert!(at(&body, "Cy") < ops, "{body}");
    assert!(!shows(&body, "Bea") && !shows(&body, "Dan"), "{body}");
}

#[test]
fn controlled_open_groups_follow_the_prop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: columns(),
                    row_grouping: team(),
                    onrowgroupingchange: |_| {},
                    expanded_groups: vec!["Ops".to_string()],
                    onexpandedgroupschange: |_| {},
                    default_expanded_groups: vec!["Sales".to_string()],
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(shows(&body, "Bea"), "{body}");
    assert!(!shows(&body, "Ada"), "the seed loses to the prop: {body}");
}

#[test]
fn default_group_expansion_opens_the_first_levels() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: columns(),
                    default_row_grouping: vec!["Team".to_string(), "City".to_string()],
                    default_group_expansion: 1,
                }
            }
        }
    }

    let body = body(&render(app));
    // Every team open, every city inside it closed.
    assert_eq!(body.matches("aria-expanded=\"true\"").count(), 3, "{body}");
    assert_eq!(body.matches("aria-expanded=\"false\"").count(), 4, "{body}");
    assert!(body.contains("aria-label=\"Berlin, 1 row\""), "{body}");
    assert!(!shows(&body, "Ada"), "{body}");
}

#[test]
fn a_page_counts_group_rows_and_does_not_repeat_a_split_group() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: columns(),
                    default_row_grouping: team(),
                    default_group_expansion: 1,
                    default_page_size: 2,
                    default_page: 2,
                }
            }
        }
    }

    let body = body(&render(app));
    // Sales, Ada | Cy, Ops | Bea, (empty) | Dan: page 2 starts inside Sales.
    assert!(shows(&body, "Cy") && shows(&body, "Ops"), "{body}");
    assert!(!body.contains("aria-label=\"Sales, 2 rows\""), "{body}");
    assert!(!shows(&body, "Ada") && !shows(&body, "Bea"), "{body}");
    assert!(
        body.contains("of 7"),
        "the pager counts the group rows: {body}"
    );
}

#[test]
fn a_filter_drops_the_groups_it_empties() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: columns(),
                    default_row_grouping: team(),
                    default_quick_filter: "Rome",
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(body.contains("aria-label=\"Sales, 1 row\""), "{body}");
    assert!(body.contains("aria-label=\"Ops, 1 row\""), "{body}");
    assert!(!body.contains("(empty)"), "{body}");
}

#[test]
fn groups_follow_the_sorted_rows() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: columns(),
                    default_row_grouping: team(),
                    default_sort: vec![TableSort::new("Name", SortDirection::Descending)],
                }
            }
        }
    }

    let body = body(&render(app));
    // Dan, Cy, Bea, Ada: (empty), Sales, Ops.
    assert!(at(&body, "(empty)") < at(&body, "Sales"), "{body}");
    assert!(at(&body, "Sales") < at(&body, "Ops"), "{body}");
}

#[test]
fn a_group_row_spans_the_lead_columns_with_one_cell() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "People",
                    data: people(),
                    columns: columns(),
                    default_row_grouping: team(),
                    selectable: true,
                    row_key: |row: &Row| row.name.to_string(),
                    row_detail: |row: &Row| Some(rsx! { "{row.city}" }),
                }
            }
        }
    }

    let body = body(&render(app));
    assert_eq!(body.matches("data-group-lead").count(), 3, "{body}");
    assert!(
        body.contains("colspan=\"2\"") || body.contains("colspan=2"),
        "{body}"
    );
    // Group rows have no checkbox of their own; select-all stays.
    assert_eq!(body.matches("type=\"checkbox\"").count(), 1, "{body}");
}

#[test]
fn a_group_row_shows_each_columns_aggregate_over_its_rows() {
    #[derive(Clone, PartialEq)]
    struct Sale {
        team: &'static str,
        amount: u32,
    }

    fn app() -> Element {
        let data = [("Sales", 3), ("Ops", 4), ("Sales", 5)]
            .map(|(team, amount)| Sale { team, amount })
            .to_vec();
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Sales",
                    data,
                    columns: vec![
                        column("Team").value(|s: &Sale| s.team.to_string()),
                        column("Amount")
                            .value(|s: &Sale| s.amount)
                            .aggregate(Aggregate::Sum)
                            .aggregate_format(|sum| format!("{sum} EUR")),
                    ],
                    default_row_grouping: team(),
                }
            }
        }
    }

    let body = body(&render(app));
    // Closed groups still sum their rows; the footer sums them all.
    assert!(at(&body, "8 EUR") < at(&body, "4 EUR"), "{body}");
    assert!(shows(&body, "12 EUR"), "{body}");
    assert_eq!(body.matches("data-group-aggregate").count(), 2, "{body}");
    assert!(
        body.contains("data-align=\"end\"") || body.contains("data-align=end"),
        "{body}"
    );
}
