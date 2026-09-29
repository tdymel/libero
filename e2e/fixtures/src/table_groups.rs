//! `Table` with column groups over a selectable, height-capped body, and a total
//! row whose label spans two columns.

use dioxus::prelude::*;
use libero::components::{PinnedColumns, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-groups", || rsx! { GroupsTablePage {} }),
    ("/table-groups/pinned", || rsx! { PinnedGroupsTablePage {} }),
];

#[derive(Clone, PartialEq)]
struct Sale {
    city: &'static str,
    region: &'static str,
    q1: u32,
    q2: u32,
}

/// Thirty sales under a 240px `max_height`, then a total row.
#[component]
fn GroupsTablePage() -> Element {
    let mut data: Vec<Sale> = (1..=30)
        .map(|n| Sale {
            city: ["Lyon", "Turin", "Graz"][n as usize % 3],
            region: ["South", "North"][n as usize % 2],
            q1: n,
            q2: n * 2,
        })
        .collect();
    data.push(Sale {
        city: "Total",
        region: "",
        q1: 465,
        q2: 930,
    });
    rsx! {
        Table {
            caption: "Sales",
            max_height: "240px",
            selectable: true,
            striped: true,
            data,
            row_key: |sale: &Sale| sale.q1.to_string(),
            columns: vec![
                column("City")
                    .value(|sale: &Sale| sale.city)
                    .row_header()
                    .group("Place")
                    .col_span(|sale: &Sale| if sale.city == "Total" { 2 } else { 1 }),
                column("Region").value(|sale: &Sale| sale.region).group("Place"),
                column("Q1").value(|sale: &Sale| sale.q1).group("Revenue").group("Half 1"),
                column("Q2").value(|sale: &Sale| sale.q2).group("Revenue").group("Half 1"),
            ],
        }
    }
}

/// A Place group pinned at the start of a table wider than its 320px box (todo 1449).
#[component]
fn PinnedGroupsTablePage() -> Element {
    let data: Vec<Sale> = (1..=4)
        .map(|n| Sale {
            city: ["Lyon", "Turin", "Graz"][n as usize % 3],
            region: ["South", "North"][n as usize % 2],
            q1: n,
            q2: n * 2,
        })
        .collect();
    rsx! {
        div { style: "width: 320px",
            Table {
                caption: "Pinned sales",
                scroll: true,
                default_pinned_columns: PinnedColumns::default().start(["City", "Region"]),
                data,
                columns: vec![
                    column("City").value(|sale: &Sale| sale.city).group("Place").width("6rem"),
                    column("Region").value(|sale: &Sale| sale.region).group("Place").width("6rem"),
                    column("Q1").value(|sale: &Sale| sale.q1).group("Revenue").min_width("10rem").sortable(),
                    column("Q2").value(|sale: &Sale| sale.q2).group("Revenue").min_width("10rem"),
                    column("Q3").value(|sale: &Sale| sale.q1 * 3).group("Revenue").min_width("10rem"),
                ],
            }
        }
    }
}
