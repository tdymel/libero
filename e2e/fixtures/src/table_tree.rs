//! `Table` of tasks nested by `parent_key`, sortable, with a quick filter; the open
//! rows printed below; a "c" typed in the table closes every row. `/table-tree-cells`
//! adds `cell_focus`.

use dioxus::prelude::*;
use libero::components::{Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-tree", || rsx! { TreePage { cells: false } }),
    ("/table-tree-cells", || rsx! { TreePage { cells: true } }),
];

#[derive(Clone, PartialEq)]
struct Task {
    id: &'static str,
    parent: &'static str,
    owner: &'static str,
}

/// site > (design > wire, build > (front, back)), launch.
fn tasks() -> Vec<Task> {
    [
        ("site", "", "Ada"),
        ("design", "site", "Bo"),
        ("wire", "design", "Bo"),
        ("build", "site", "Di"),
        ("front", "build", "Di"),
        ("back", "build", "Ed"),
        ("launch", "", "Ada"),
    ]
    .map(|(id, parent, owner)| Task { id, parent, owner })
    .to_vec()
}

#[component]
fn TreePage(cells: bool) -> Element {
    let mut expanded = use_signal(Vec::<String>::new);
    rsx! {
        div {
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Character("c".into()) {
                    expanded.set(Vec::new());
                }
            },
            Table {
            aria_label: "Tasks",
            show_quick_filter: true,
            data: tasks(),
            columns: vec![
                column("Task").value(|t: &Task| t.id.to_string()).sortable().row_header(),
                column("Owner").value(|t: &Task| t.owner.to_string()),
            ],
            row_key: |t: &Task| t.id.to_string(),
            parent_key: |t: &Task| (!t.parent.is_empty()).then(|| t.parent.to_string()),
            expanded_nodes: expanded(),
            onexpandednodeschange: move |next| expanded.set(next),
            cell_focus: cells,
            }
        }
        button { id: "after", "After" }
        p { id: "expanded", {expanded.read().join(",")} }
    }
}
