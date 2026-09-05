use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Tree, TreeItem, TreeNode, TreeNodeRenderArgs},
};

/// The row hands its content the tab stop and the `disabled` flag through
/// context, so a `TreeItem` needs neither at the call site.
#[test]
fn a_tree_item_takes_its_tabindex_from_the_row() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data: vec![TreeNode::new("a", "Alpha".to_string())],
                    render_node: move |_: TreeNodeRenderArgs<String>| rsx! {
                        TreeItem { "Alpha" }
                    },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<button"));
    assert!(body.contains(r#"tabindex="-1""#));
}

#[test]
fn tree_renders_a_labelled_row_per_node() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string()),
            TreeNode::new("b", "Beta".to_string()),
        ];

        rsx! {
            LiberoProvider {
                Tree { aria_label: "Files", data }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(attributes_of(&html, "ul")["aria-label"], "Files");
    assert!(body.contains("Alpha"));
    assert!(body.contains("Beta"));
}

/// The tab stop starts on `current` - a nav's current page - rather than on
/// the first row.
#[test]
fn the_tab_stop_starts_on_the_current_node() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string()),
            TreeNode::new("b", "Beta".to_string()),
        ];

        rsx! {
            LiberoProvider {
                Tree { aria_label: "Files", data, current: "b" }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let tab_stops: Vec<&str> = body
        .split("<li")
        .filter(|row| row.contains(r#"tabindex="0""#))
        .collect();

    assert_eq!(tab_stops.len(), 1);
    assert!(tab_stops[0].contains("Beta"));
}

/// A `current` hidden in a collapsed branch hands the tab stop to the branch.
#[test]
fn a_collapsed_current_node_puts_the_tab_stop_on_its_branch() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string()),
            TreeNode::new("b", "Beta".to_string())
                .children(vec![TreeNode::new("c", "Gamma".to_string())]),
        ];

        rsx! {
            LiberoProvider {
                Tree { aria_label: "Files", data, current: "c" }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let tab_stops: Vec<&str> = body
        .split("<li")
        .filter(|row| row.contains(r#"tabindex="0""#))
        .collect();

    assert_eq!(tab_stops.len(), 1);
    assert!(tab_stops[0].contains("Beta"));
}
