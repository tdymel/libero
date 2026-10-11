//! `Table` grouped by team, with a checkbox column and an Age sum in each group
//! row; the open groups printed below.
//! `/menu`: ungrouped, grouped from the column menu, the grouping printed below.

use dioxus::prelude::*;
use libero::components::{Aggregate, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-row-groups", || rsx! { RowGroupsPage {} }),
    ("/table-row-groups/menu", || rsx! { GroupMenuPage {} }),
];

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    team: &'static str,
    age: u32,
}

fn people() -> Vec<Person> {
    [("Ada", "Sales", 30), ("Bea", "Ops", 41), ("Cy", "Sales", 25), ("Dan", "", 52)]
        .map(|(name, team, age)| Person { name, team, age })
        .to_vec()
}

#[component]
fn RowGroupsPage() -> Element {
    let mut expanded = use_signal(Vec::<String>::new);
    rsx! {
        Table {
            aria_label: "People",
            selectable: true,
            striped: true,
            data: people(),
            columns: vec![
                column("Name").value(|p: &Person| p.name.to_string()).sortable().row_header(),
                column("Team").value(|p: &Person| p.team.to_string()),
                column("Age").value(|p: &Person| p.age).aggregate(Aggregate::Sum),
            ],
            row_key: |p: &Person| p.name.to_string(),
            default_row_grouping: vec!["Team".to_string()],
            expanded_groups: expanded(),
            onexpandedgroupschange: move |next| expanded.set(next),
        }
        p { id: "expanded", {expanded.read().join(",")} }
    }
}

#[component]
fn GroupMenuPage() -> Element {
    let mut grouping = use_signal(Vec::<String>::new);
    rsx! {
        Table {
            aria_label: "People",
            column_menu: true,
            data: people(),
            columns: vec![
                column("Name")
                    .value(|p: &Person| p.name.to_string())
                    .row_header()
                    .row_groupable(false),
                column("Team").value(|p: &Person| p.team.to_string()),
            ],
            row_key: |p: &Person| p.name.to_string(),
            onrowgroupingchange: move |next| grouping.set(next),
        }
        p { id: "grouping", {grouping.read().join(",")} }
    }
}
