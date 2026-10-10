//! `Table` grouped by team, with a checkbox column; the open groups printed below.

use dioxus::prelude::*;
use libero::components::{Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[("/table-row-groups", || rsx! { RowGroupsPage {} })];

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    team: &'static str,
}

fn people() -> Vec<Person> {
    [("Ada", "Sales"), ("Bea", "Ops"), ("Cy", "Sales"), ("Dan", "")]
        .map(|(name, team)| Person { name, team })
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
            ],
            row_key: |p: &Person| p.name.to_string(),
            default_row_grouping: vec!["Team".to_string()],
            expanded_groups: expanded(),
            onexpandedgroupschange: move |next| expanded.set(next),
        }
        p { id: "expanded", {expanded.read().join(",")} }
    }
}
