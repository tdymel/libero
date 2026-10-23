//! `Tree` in Blitz: a row starts at its edge and the painted chevron turns.
//! Keys and the computed turn are shared scenarios (`tree::`).

use std::collections::HashSet;

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{Tree, TreeItem, TreeNode, TreeNodeRenderArgs};

const CHEVRON: &str = "[role=treeitem][aria-expanded] [data-tree-chevron]";

fn app() -> Element {
    let data = vec![
        TreeNode::new("src", "src").children(vec![TreeNode::new("src/lib.rs", "lib.rs")]),
        TreeNode::new("README.md", "README.md"),
    ];
    rsx! {
        Tree {
            aria_label: "Files",
            data,
        }
    }
}

/// Blitz's UA sheet centres a button's content; a row starts at its edge.
#[test]
fn a_row_starts_its_content_at_the_edge() {
    let page = mount(|| {
        rsx! {
            Tree {
                aria_label: "Files",
                data: vec![TreeNode::new("README.md", "README.md")],
                render_node: |args: TreeNodeRenderArgs<&'static str>| rsx! {
                    TreeItem { span { "{args.data}" } }
                },
            }
        }
    });
    let row = page.rect("[role=treeitem] [data-tree-activate]");
    let first = page.rect("[role=treeitem] [data-tree-activate] > *");
    assert_eq!(
        first.0,
        row.0,
        "row {row:?}, first child {first:?}\n{}",
        page.tree()
    );
}

/// Review 449 natively: Blitz focuses a clicked row button after the click's
/// handlers, so the tree took focus back too early and the arrows were dead.
#[test]
fn the_arrows_work_after_a_click_on_a_row_button() {
    let mut page = mount(|| {
        rsx! {
            Tree {
                aria_label: "Files",
                data: vec![TreeNode::new("src", "src"), TreeNode::new("README.md", "README.md")],
                render_node: |args: TreeNodeRenderArgs<&'static str>| rsx! {
                    TreeItem { "{args.data}" }
                },
            }
        }
    });
    page.click("[data-tree-id='README.md'] [data-tree-activate]");
    assert!(
        page.is_focused("[data-tree-id='README.md']"),
        "focus on {}",
        page.focus_owner()
    );
    page.press(Key::Named(NamedKey::ArrowUp));
    assert!(
        page.is_focused("[data-tree-id='src']"),
        "focus on {}",
        page.focus_owner()
    );
}

/// Blitz's click on a link navigates but leaves focus where it was; the web
/// focuses the link, which the row then takes back.
#[test]
fn the_arrows_work_after_a_click_on_a_row_link() {
    let mut page = mount(|| {
        rsx! {
            Tree {
                aria_label: "Files",
                data: vec![TreeNode::new("src", "src"), TreeNode::new("README.md", "README.md")],
                render_node: |args: TreeNodeRenderArgs<&'static str>| rsx! {
                    a { href: "#{args.data}", "{args.data}" }
                },
            }
        }
    });
    page.click("[data-tree-id='README.md'] a");
    assert!(
        page.is_focused("[data-tree-id='README.md']"),
        "focus on {}",
        page.focus_owner()
    );
    page.press(Key::Named(NamedKey::ArrowUp));
    assert!(
        page.is_focused("[data-tree-id='src']"),
        "focus on {}",
        page.focus_owner()
    );
}

/// Todo 3975: a checked row's box fills and paints its mark, an unchecked one stays
/// empty, and a selected row paints its tint and inset line.
#[test]
fn a_checked_box_and_a_selected_row_paint_their_look() {
    let page = mount(|| {
        rsx! {
            Tree {
                aria_label: "Files",
                data: vec![
                    TreeNode::new("a", "Alpha"),
                    TreeNode::new("b", "Beta"),
                    TreeNode::new("c", "Gamma"),
                ],
                checked: HashSet::from(["a".to_string()]),
                check_strictly: true,
                oncheckedchange: |_| {},
                selected: HashSet::from(["c".to_string()]),
                onselectedchange: |_| {},
            }
        }
    });
    // A pixel inside the left edge, clear of the mark, at mid height; and the top edge's middle.
    let edge = |selector: &str, inset: f64| {
        let (x, y, width, height) = page.rect(selector);
        (
            page.painted_pixel((x + inset) as u32, (y + height / 2.0) as u32),
            page.painted_pixel((x + width / 2.0) as u32, y as u32),
        )
    };
    let checked = "[data-tree-id=a] [data-slot=box]";
    assert_eq!(
        edge(checked, 3.0).0,
        page.computed(checked, "background-color"),
        "the checked box's fill"
    );
    assert_eq!(
        page.painted_stroke(&format!("{checked} > svg")),
        page.computed(checked, "color"),
        "the checked box's mark"
    );
    let unchecked = "[data-tree-id=b] [data-slot=box]";
    assert_eq!(
        edge(unchecked, 0.5).0,
        page.computed(unchecked, "border-left-color"),
        "the unchecked box's border"
    );
    assert_eq!(
        edge(unchecked, 3.0).0,
        "rgb(255, 255, 255)",
        "the unchecked box's inside"
    );
    let selected = "[data-tree-id=c] [data-slot=content]";
    let (tint, line) = edge(selected, 40.0);
    assert_eq!(
        tint,
        page.computed(selected, "background-color"),
        "the selected row's tint"
    );
    let shadow = page.computed(selected, "box-shadow");
    assert!(
        shadow.starts_with(&line),
        "the selected row's inset line {line}, shadow {shadow}"
    );
    assert_eq!(
        edge("[data-tree-id=b] [data-slot=content]", 40.0).0,
        "rgb(255, 255, 255)"
    );
}

#[test]
fn the_painted_chevron_turns_too() {
    let mut page = mount(app);
    let collapsed = page.painted_transform(CHEVRON);
    page.focus("[tabindex='0']");
    page.press(Key::Named(NamedKey::ArrowRight));
    page.advance(1.0);
    let expanded = page.painted_transform(CHEVRON);
    assert_ne!(
        collapsed, expanded,
        "the painted chevron stayed at {collapsed:?}"
    );
}
