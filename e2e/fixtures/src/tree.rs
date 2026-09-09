//! `Tree`.

use dioxus::prelude::*;
use libero::components::{Flex, Tree, TreeItem, TreeNode, TreeNodeRenderArgs};

use crate::Routes;

pub const ROUTES: Routes = &[("/tree", || rsx! { TreePage {} })];

/// A deliberately awkward fixture: `Tree` is listed under the roving-tabindex
/// archetype, and this exists to find out whether that claim survives contact.
/// A tree is roving *and* hierarchical - it has expansion, levels, and rows
/// that appear and disappear - none of which a flat strip has.
#[component]
fn TreePage() -> Element {
    let data = vec![
        TreeNode::new("src", "src").children(vec![
            TreeNode::new("src/lib.rs", "lib.rs"),
            TreeNode::new("src/main.rs", "main.rs"),
        ]),
        TreeNode::new("docs", "docs").children(vec![TreeNode::new("docs/index.md", "index.md")]),
        TreeNode::new("README.md", "README.md"),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Tree {
                aria_label: "Project files",
                data,
                // Collapsed on purpose: an expandable row that is closed is the
                // case a flat archetype cannot see.
                render_node: move |args: TreeNodeRenderArgs<&'static str>| {
                    rsx! {
                        TreeItem { tabindex: args.tabindex, "{args.data}" }
                    }
                },
            }
        }
    }
}
