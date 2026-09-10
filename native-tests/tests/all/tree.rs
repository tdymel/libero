//! `Tree`'s chevron turns by a state-driven `transform` when a row expands.

use dioxus::prelude::*;
use libero::components::{Tree, TreeNode};
use native_tests::{Key, mount};

const BRANCH: &str = "[role=treeitem][aria-expanded]";
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

#[test]
fn the_chevron_turns_when_the_branch_expands() {
    let mut page = mount(app);
    let collapsed = page.computed(CHEVRON, "transform");
    page.focus("[tabindex='0']");
    page.press(Key::ArrowRight);
    assert_eq!(
        page.attr(BRANCH, "aria-expanded").as_deref(),
        Some("true"),
        "{}",
        page.tree()
    );
    page.advance(1.0);
    let expanded = page.computed(CHEVRON, "transform");
    assert_ne!(collapsed, expanded, "the chevron stayed at {collapsed}");
}
