//! `Table` of tasks nested by `parent_key`, sortable, with a quick filter; the open
//! rows printed below; a "c" typed in the table closes every row. `/table-tree-cells`
//! adds `cell_focus`. `/table-tree-select` selects rows, "launch" loading its
//! children on "Load"; `/table-tree-window` windows 200 rows, an "x" closing the first root.

use dioxus::prelude::*;
use libero::components::{Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-tree", || rsx! { TreePage { cells: false } }),
    ("/table-tree-cells", || rsx! { TreePage { cells: true } }),
    ("/table-tree-select", || rsx! { TreeSelectPage {} }),
    ("/table-tree-window", || rsx! { TreeWindowPage {} }),
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

#[component]
fn TreeSelectPage() -> Element {
    let mut expanded = use_signal(|| vec!["site".to_string()]);
    let mut selection = use_signal(Vec::<String>::new);
    let mut loaded = use_signal(|| false);
    let mut rows = tasks();
    if loaded() {
        rows.extend([("ship", "launch", "Cy"), ("tell", "launch", "Ed")].map(
            |(id, parent, owner)| Task { id, parent, owner },
        ));
    }
    let mut picked = selection();
    picked.sort();
    rsx! {
        Table {
            aria_label: "Tasks",
            data: rows,
            columns: vec![
                column("Task").value(|t: &Task| t.id.to_string()).row_header(),
                column("Owner").value(|t: &Task| t.owner.to_string()),
            ],
            row_key: |t: &Task| t.id.to_string(),
            parent_key: |t: &Task| (!t.parent.is_empty()).then(|| t.parent.to_string()),
            row_has_children: |t: &Task| t.id == "launch",
            expanded_nodes: expanded(),
            onexpandednodeschange: move |next| expanded.set(next),
            selectable: true,
            selection: selection(),
            onselectionchange: move |next| selection.set(next),
        }
        button { id: "load", onclick: move |_| loaded.set(true), "Load" }
        p { id: "selection", {picked.join(",")} }
        p { id: "expanded", {expanded.read().join(",")} }
    }
}

#[derive(Clone, PartialEq)]
struct Node {
    id: String,
    parent: Option<String>,
}

/// 40 roots `r0`..`r39` of 4 children each, `r0 c0`..`r0 c3`.
fn nodes() -> Vec<Node> {
    (0..40)
        .flat_map(|root| {
            let id = format!("r{root}");
            let children = (0..4).map(move |child| Node {
                id: format!("r{root} c{child}"),
                parent: Some(format!("r{root}")),
            });
            std::iter::once(Node { id, parent: None }).chain(children)
        })
        .collect()
}

#[component]
fn TreeWindowPage() -> Element {
    let mut expanded = use_signal(|| (0..40).map(|root| format!("r{root}")).collect::<Vec<_>>());
    rsx! {
        div {
            // An "x" closes r0, far above the window's end.
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Character("x".into()) {
                    expanded.write().retain(|key| key != "r0");
                }
            },
            Table {
                aria_label: "Nodes",
                data: nodes(),
                columns: vec![column("Node").value(|n: &Node| n.id.clone()).row_header()],
                row_key: |n: &Node| n.id.clone(),
                parent_key: |n: &Node| n.parent.clone(),
                expanded_nodes: expanded(),
                onexpandednodeschange: move |next| expanded.set(next),
                max_height: "240px",
                virtual_row_height: 36.0,
            }
        }
    }
}
