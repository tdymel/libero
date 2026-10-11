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
    // The row and its toggle.
    assert_eq!(body.matches("aria-expanded=\"false\"").count(), 2, "{body}");
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
    // Two levels open: site and its children, not the grandchildren (they have none). Row and toggle each.
    assert_eq!(body.matches("aria-expanded=\"true\"").count(), 6, "{body}");
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

/// The opening tag of the row whose first cell reads `text`.
fn row_of<'a>(body: &'a str, text: &str) -> &'a str {
    let start = body[..at(body, text)].rfind("<tr").expect("a row");
    let end = start + body[start..].find('>').expect("a tag");
    &body[start..end]
}

#[test]
fn a_tree_is_a_treegrid_whose_rows_tell_their_place() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Tasks",
                    data: tasks(),
                    columns: columns(),
                    row_key: |t: &Task| t.id.to_string(),
                    parent_key: parent,
                    default_expanded_nodes: vec!["site".to_string()],
                }
            }
        }
    }

    let html = render(app);
    assert!(html.contains("role=\"treegrid\""), "{html}");
    let body = body(&html);
    let site = row_of(&body, "site");
    for part in [
        "aria-level=\"1\"",
        "aria-posinset=\"1\"",
        "aria-setsize=\"3\"",
        "aria-expanded=\"true\"",
        "tabindex=\"0\"",
    ] {
        assert!(site.contains(part), "{part}: {site}");
    }
    let build = row_of(&body, "build");
    for part in [
        "aria-level=\"2\"",
        "aria-posinset=\"2\"",
        "aria-setsize=\"2\"",
        "aria-expanded=\"false\"",
        "tabindex=\"-1\"",
    ] {
        assert!(build.contains(part), "{part}: {build}");
    }
    // A leaf has no `aria-expanded`; the toggles leave Tab to the rows.
    assert!(!row_of(&body, "launch").contains("aria-expanded"), "{body}");
    assert_eq!(body.matches("tabindex=\"0\"").count(), 1, "{body}");
    let toggle = &body[body[..body.find("data-node-toggle").expect("a toggle")]
        .rfind("<button")
        .expect("a button")..];
    let toggle = &toggle[..toggle.find('>').expect("a tag")];
    assert!(toggle.contains("tabindex=\"-1\""), "{toggle}");
}

#[test]
fn cell_focus_leaves_the_first_stop_on_the_header() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Table {
                    caption: "Tasks",
                    data: tasks(),
                    columns: columns(),
                    row_key: |t: &Task| t.id.to_string(),
                    parent_key: parent,
                    cell_focus: true,
                }
            }
        }
    }

    let html = render(app);
    assert!(html.contains("role=\"treegrid\""), "{html}");
    let body = &html[html.find("<tbody").expect("a body")..];
    assert!(!body.contains("tabindex=\"0\""), "{body}");
    assert!(row_of(body, "site").contains("tabindex=\"-1\""), "{body}");
}
