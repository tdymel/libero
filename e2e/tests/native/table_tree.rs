//! `Table` tree rows on Blitz (1156 TR1): a row's toggle opens its children under
//! it, indented one step past it; Enter on the focused toggle closes it again. A
//! treegrid (TR2): its role, and the arrows on a focused row.

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{Table, column};

#[derive(Clone, PartialEq)]
struct Task {
    id: &'static str,
    parent: &'static str,
}

fn app() -> Element {
    let tasks = [
        ("site", ""),
        ("design", "site"),
        ("build", "site"),
        ("front", "build"),
        ("launch", ""),
    ]
    .map(|(id, parent)| Task { id, parent })
    .to_vec();
    rsx! {
        div { width: "400px",
            Table {
                aria_label: "Tasks",
                data: tasks,
                columns: vec![column("Task").value(|t: &Task| t.id.to_string()).row_header()],
                row_key: |t: &Task| t.id.to_string(),
                parent_key: |t: &Task| (!t.parent.is_empty()).then(|| t.parent.to_string()),
            }
        }
    }
}

const SITE: &str = "button[aria-label=\"site, 2 rows\"]";

#[test]
fn a_tree_toggle_opens_its_children_under_it() {
    let mut page = mount(app);
    assert_eq!(page.query_all("tbody tr").len(), 2, "{}", page.tree());
    // An inline box in a padded cell takes no hit on Blitz; the toggle's wrapper avoids it.
    assert!(page.hits(SITE), "{}", page.tree());
    page.click(SITE);
    assert_eq!(page.attr(SITE, "aria-expanded").as_deref(), Some("true"));
    assert_eq!(page.query_all("tbody tr").len(), 4, "{}", page.tree());
    assert_eq!(page.text("tbody tr:nth-child(3) th").trim(), "build");
    let (parent_x, ..) = page.rect(SITE);
    let (child_x, ..) = page.rect("button[aria-label=\"build, 1 row\"]");
    assert!(
        child_x > parent_x,
        "the child's toggle ({child_x}) is not indented past its parent's ({parent_x})"
    );
    page.focus(SITE);
    page.press(Key::Enter);
    assert_eq!(page.query_all("tbody tr").len(), 2, "{}", page.tree());
}

#[test]
fn a_tree_is_a_treegrid_in_the_ax_tree() {
    let page = mount(app);
    for (selector, role) in [
        ("table", "TreeGrid"),
        ("tbody tr", "Row"),
        ("tbody th", "RowHeader"),
    ] {
        assert_eq!(
            page.accessible(selector).0,
            role,
            "{selector}: {}",
            page.tree()
        );
    }
}

/// Blitz fires no `focusin` on a `focus()`: the keys find the row by its `data-cell`.
#[test]
fn the_arrows_open_and_close_a_focused_row() {
    let mut page = mount(app);
    page.focus("tbody tr:nth-child(1)");
    page.press(Key::ArrowRight);
    assert_eq!(page.query_all("tbody tr").len(), 4, "{}", page.tree());
    page.press(Key::ArrowDown);
    page.press(Key::ArrowRight);
    assert!(
        page.is_focused("tbody tr:nth-child(2)"),
        "{}",
        page.focus_owner()
    );
    page.press(Key::ArrowLeft);
    assert!(
        page.is_focused("tbody tr:nth-child(1)"),
        "{}",
        page.focus_owner()
    );
    page.press(Key::ArrowLeft);
    assert_eq!(page.query_all("tbody tr").len(), 2, "{}", page.tree());
}
