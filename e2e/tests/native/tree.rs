//! `Tree` in Blitz: a row starts at its edge and the painted chevron turns.
//! Keys and the computed turn are shared scenarios (`tree::`).

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
    let row = page.rect("[role=treeitem] button");
    let first = page.rect("[role=treeitem] button > *");
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
    page.click("[data-tree-id='README.md'] button");
    assert!(
        page.is_focused("[data-tree-id='README.md']"),
        "focus on {}",
        page.focus_owner()
    );
    page.press(Key::ArrowUp);
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
    page.press(Key::ArrowUp);
    assert!(
        page.is_focused("[data-tree-id='src']"),
        "focus on {}",
        page.focus_owner()
    );
}

#[test]
fn the_painted_chevron_turns_too() {
    let mut page = mount(app);
    let collapsed = page.painted_transform(CHEVRON);
    page.focus("[tabindex='0']");
    page.press(Key::ArrowRight);
    page.advance(1.0);
    let expanded = page.painted_transform(CHEVRON);
    assert_ne!(
        collapsed, expanded,
        "the painted chevron stayed at {collapsed:?}"
    );
}
