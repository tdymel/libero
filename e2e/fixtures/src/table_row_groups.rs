//! `Table` grouped by team, with a checkbox column and an Age sum in each group
//! row; the open groups printed below.
//! `/menu`: ungrouped, grouped from the column menu, the grouping printed below.
//! `/windowed`: 10k rows in four open groups under a 320px cap, 40px rows, selectable.
//! `/held`: 200 rows in four open groups, windowed so; an "x" closes Sales, the first.

use dioxus::prelude::*;
use libero::components::{Aggregate, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-row-groups", || rsx! { RowGroupsPage {} }),
    ("/table-row-groups/menu", || rsx! { GroupMenuPage {} }),
    ("/table-row-groups/windowed", || rsx! { WindowedGroupsPage {} }),
    ("/table-row-groups/held", || rsx! { HeldGroupsPage {} }),
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
    let mut selection = use_signal(Vec::<String>::new);
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
            onselectionchange: move |next| selection.set(next),
        }
        p { id: "expanded", {expanded.read().join(",")} }
        p { id: "selection", {selection.read().join(",")} }
    }
}

#[derive(Clone, PartialEq)]
struct Member {
    name: String,
    team: &'static str,
}

const TEAMS: [&str; 4] = ["Sales", "Ops", "Legal", "Labs"];

#[component]
fn WindowedGroupsPage() -> Element {
    let mut selection = use_signal(Vec::<String>::new);
    let members: Vec<Member> = (0..10_000)
        .map(|at| Member {
            name: format!("Person {at}"),
            team: TEAMS[at % TEAMS.len()],
        })
        .collect();
    rsx! {
        Table {
            aria_label: "Members",
            selectable: true,
            max_height: "320px",
            virtual_row_height: 40.0,
            data: members,
            columns: vec![
                column("Name").value(|m: &Member| m.name.clone()).sortable().row_header(),
                column("Team").value(|m: &Member| m.team.to_string()),
            ],
            row_key: |m: &Member| m.name.clone(),
            default_row_grouping: vec!["Team".to_string()],
            default_group_expansion: 1,
            onselectionchange: move |next: Vec<String>| selection.set(next),
        }
        p { id: "selection", "{selection.read().len()}" }
    }
}

#[component]
fn HeldGroupsPage() -> Element {
    let mut expanded = use_signal(|| TEAMS.map(String::from).to_vec());
    let members: Vec<Member> = (0..200)
        .map(|at| Member {
            name: format!("Person {at}"),
            team: TEAMS[at / 50],
        })
        .collect();
    rsx! {
        div {
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Character("x".into()) {
                    expanded.write().retain(|id| id != "Sales");
                }
            },
            Table {
                aria_label: "Members",
                selectable: true,
                max_height: "320px",
                virtual_row_height: 40.0,
                data: members,
                columns: vec![
                    column("Name").value(|m: &Member| m.name.clone()).row_header(),
                    column("Team").value(|m: &Member| m.team.to_string()),
                ],
                row_key: |m: &Member| m.name.clone(),
                default_row_grouping: vec!["Team".to_string()],
                expanded_groups: expanded(),
                onexpandedgroupschange: move |next| expanded.set(next),
            }
        }
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
