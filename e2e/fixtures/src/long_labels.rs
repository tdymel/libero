//! One label with no break opportunity in each of `Tree` (default row and
//! `TreeItem`), `NavLink` and `Menu`, for reflow at a narrow width (todo 518).

use dioxus::prelude::*;
use libero::components::{
    Button, Flex, Menu, MenuItem, NavLink, Tree, TreeItem, TreeNode, TreeNodeRenderArgs, use_menu,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/long-labels", || rsx! { LongLabelsPage {} })];

const LONG: &str = "Versandkostenberechnungsgrundlagenverordnungsentwurfsbearbeitungsstelle";

#[component]
fn LongLabelsPage() -> Element {
    let menu = use_menu();
    let nodes = || {
        vec![
            TreeNode::new("docs", "docs").children(vec![TreeNode::new("docs/long", LONG)]),
            TreeNode::new("long", LONG),
        ]
    };
    let expanded = || ["docs".to_string()].into_iter().collect();

    rsx! {
        Flex { direction: "column", gap: "md",
            Tree {
                id: "tree-default",
                aria_label: "Default rows",
                data: nodes(),
                default_expanded: expanded(),
            }
            Tree {
                id: "tree-item",
                aria_label: "TreeItem rows",
                data: nodes(),
                default_expanded: expanded(),
                render_node: move |args: TreeNodeRenderArgs<&'static str>| rsx! {
                    TreeItem { tabindex: args.tabindex, "{args.data}" }
                },
            }
            nav { id: "nav",
                NavLink { to: "https://example.com", active: true, "{LONG}" }
            }
            Menu {
                state: menu,
                items: vec![MenuItem::new("Short").into(), MenuItem::new(LONG).into()],
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Open" }
            }
        }
    }
}
