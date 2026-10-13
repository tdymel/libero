//! `Table` tree rows on Blitz (1156 TR1): a row's toggle opens its children under
//! it, indented one step past it; Enter on the focused toggle closes it again. A
//! treegrid (TR2): its role, and the arrows on a focused row.

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
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

fn selectable() -> Element {
    let tasks = [
        ("site", ""),
        ("design", "site"),
        ("build", "site"),
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
                default_expanded_nodes: vec!["site".to_string()],
                selectable: true,
            }
        }
    }
}

/// The pixel at `at` of `selector`'s box, as fractions of its size.
fn pixel(page: &Page, selector: &str, at: f64) -> String {
    let (x, y, width, height) = page.rect(selector);
    page.painted_pixel((x + width * at) as u32, (y + height * at) as u32)
}

fn select_box(row: usize) -> String {
    format!("tbody tr:nth-child({row}) td[data-select] span[aria-hidden]")
}

#[test]
fn a_mixed_tree_box_paints_its_dash() {
    let mut page = mount(selectable);
    // Rows: site, design, build, launch.
    // A corner the marks miss: the fill alone.
    let empty = pixel(&page, &select_box(1), 0.25);
    page.click(&select_box(2));
    let aria = "tbody tr:nth-child(1) td[data-select] input";
    assert_eq!(
        page.attr(aria, "aria-checked").as_deref(),
        Some("mixed"),
        "{}",
        page.tree()
    );
    // Past the fill's 150ms transition delay, which Blitz 0.3.0-beta.3 runs.
    page.wait(Duration::from_millis(300));
    let fill = pixel(&page, &select_box(1), 0.25);
    assert_ne!(fill, empty, "the mixed box paints like an empty one");
    // The middle is on the dash and off the check.
    let (mixed, checked) = (
        pixel(&page, &select_box(1), 0.5),
        pixel(&page, &select_box(2), 0.5),
    );
    assert_ne!(mixed, checked, "the mixed box paints like a checked one");
}

/// 3290: a focused row's ring sits on the `tr`, which Blitz may not paint.
#[test]
fn a_focused_tree_row_paints_its_ring() {
    let mut page = mount(app);
    let (x, y, width, height) = page.rect("tbody tr:nth-child(1)");
    let row: Vec<(u32, u32)> = (y as u32..(y + height) as u32)
        .flat_map(|py| (x as u32..(x + width) as u32).map(move |px| (px, py)))
        .collect();
    let before = page.painted_pixels(&row);
    page.tab();
    assert!(
        page.is_focused("tbody tr:nth-child(1)"),
        "{}",
        page.focus_owner()
    );
    let shadow = page.computed("tbody tr:nth-child(1)", "box-shadow");
    let after = page.painted_pixels(&row);
    assert_ne!(
        before, after,
        "no ring painted over the row; box-shadow {shadow}"
    );
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
