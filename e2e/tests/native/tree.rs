//! `Tree` as Blitz lays it out and paints it: a row starts at its edge, and the
//! painted chevron turns. The keys and the computed turn are e2e's shared
//! scenarios (`tree::`).

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
