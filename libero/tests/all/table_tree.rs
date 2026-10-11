use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Column, SortDirection, Table, TableSort, column},
};

#[derive(Clone, PartialEq)]
struct Task {
    id: &'static str,
    parent: &'static str,
    owner: &'static str,
}

/// site > (design > wire, build > (front, back)), launch; "lost" names no row.
fn tasks() -> Vec<Task> {
    [
        ("site", "", "Ada"),
        ("design", "site", "Bo"),
        ("wire", "design", "Bo"),
        ("build", "site", "Di"),
        ("front", "build", "Di"),
        ("back", "build", "Ed"),
        ("launch", "", "Ada"),
        ("lost", "gone", "Cy"),
    ]
    .map(|(id, parent, owner)| Task { id, parent, owner })
    .to_vec()
}

fn columns() -> Vec<Column<Task>> {
    vec![
        column("Task")
            .value(|t: &Task| t.id.to_string())
            .sortable()
            .row_header(),
        column("Owner").value(|t: &Task| t.owner.to_string()),
    ]
}

fn parent(task: &Task) -> Option<String> {
    (!task.parent.is_empty()).then(|| task.parent.to_string())
}

/// Where `text` starts as a cell's whole text.
fn at(body: &str, text: &str) -> usize {
    body.find(&format!(">{text}<"))
        .unwrap_or_else(|| panic!("no {text}: {body}"))
}

fn shows(body: &str, text: &str) -> bool {
    body.contains(&format!(">{text}<"))
}

#[test]
fn a_tree_starts_with_its_roots_closed() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Tasks",
                    data: tasks(),
                    columns: columns(),
                    row_key: |t: &Task| t.id.to_string(),
                    parent_key: parent,
                }
            }
        }
    }

    let body = body(&render(app));
    for root in ["site", "launch", "lost"] {
        assert!(shows(&body, root), "{root}: {body}");
    }
    for child in ["design", "wire", "build", "front", "back"] {
        assert!(
            !shows(&body, child),
            "{child} is under a closed row: {body}"
        );
    }
    // Named as a group toggle is, its state in `aria-expanded`.
    assert!(body.contains("aria-label=\"site, 2 rows\""), "{body}");
    assert_eq!(body.matches("data-node-toggle").count(), 1, "{body}");
    assert_eq!(body.matches("aria-expanded=\"false\"").count(), 1, "{body}");
    assert_eq!(body.matches("data-node-spacer").count(), 2, "{body}");
}

#[test]
fn open_rows_show_their_children_after_them_in_the_sort() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Tasks",
                    data: tasks(),
                    columns: columns(),
                    row_key: |t: &Task| t.id.to_string(),
                    parent_key: parent,
                    default_node_expansion: 2,
                    default_sort: vec![TableSort::new("Task", SortDirection::Descending)],
                }
            }
        }
    }

    let body = body(&render(app));
    // Descending per level: site > (design > wire, build > (front, back)), then lost, launch.
    let order = [
        "site", "design", "wire", "build", "front", "back", "lost", "launch",
    ];
    let places: Vec<usize> = order.iter().map(|text| at(&body, text)).collect();
    assert!(places.is_sorted(), "{order:?} out of order: {body}");
    assert!(
        body.contains("data-level=\"2\"") || body.contains("data-level=2"),
        "{body}"
    );
    // Two levels open: site and its children, not the grandchildren (they have none).
    assert_eq!(body.matches("aria-expanded=\"true\"").count(), 3, "{body}");
}

#[test]
fn controlled_open_rows_follow_the_prop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Tasks",
                    data: tasks(),
                    columns: columns(),
                    row_key: |t: &Task| t.id.to_string(),
                    parent_key: parent,
                    expanded_nodes: vec!["site".to_string()],
                    onexpandednodeschange: |_| {},
                    default_expanded_nodes: vec!["build".to_string()],
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(shows(&body, "design") && shows(&body, "build"), "{body}");
    assert!(!shows(&body, "front"), "the seed loses to the prop: {body}");
}

#[test]
fn a_filter_keeps_and_opens_the_ancestors_of_a_match() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Tasks",
                    data: tasks(),
                    columns: columns(),
                    row_key: |t: &Task| t.id.to_string(),
                    parent_key: parent,
                    default_quick_filter: "front",
                }
            }
        }
    }

    let body = body(&render(app));
    for shown in ["site", "build", "front"] {
        assert!(shows(&body, shown), "{shown}: {body}");
    }
    for hidden in ["design", "back", "launch"] {
        assert!(!shows(&body, hidden), "{hidden}: {body}");
    }
    assert!(at(&body, "site") < at(&body, "build") && at(&body, "build") < at(&body, "front"));
}

#[test]
fn a_tree_ignores_the_row_grouping() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Tasks",
                    data: tasks(),
                    columns: columns(),
                    row_key: |t: &Task| t.id.to_string(),
                    parent_key: parent,
                    default_row_grouping: vec!["Owner".to_string()],
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(!body.contains("data-row-group"), "{body}");
    assert!(body.contains("data-node-toggle"), "{body}");
}
