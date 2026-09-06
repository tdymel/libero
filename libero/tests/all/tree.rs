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

/// Two nodes with one id used to make two tab stops in one composite. Like
/// `Tabs` with a repeated value, the first match wins: the tab stop is placed
/// by position, so exactly one row is `tabindex="0"`, and it is the first.
#[test]
fn a_repeated_id_still_makes_one_tab_stop_on_its_first_row() {
    fn siblings() -> Element {
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data: vec![
                        TreeNode::new("a", "First".to_string()),
                        TreeNode::new("a", "Second".to_string()),
                        TreeNode::new("b", "Third".to_string()),
                    ],
                }
            }
        }
    }
    // The same id in two open branches, and `current` naming it.
    fn cousins() -> Element {
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data: vec![
                        TreeNode::new("src", "src".to_string())
                            .children(vec![TreeNode::new("mod", "First".to_string())]),
                        TreeNode::new("lib", "lib".to_string())
                            .children(vec![TreeNode::new("mod", "Second".to_string())]),
                    ],
                    default_expanded: ["src".to_string(), "lib".to_string()].into(),
                    current: "mod".to_string(),
                }
            }
        }
    }

    for (app, first) in [(siblings as fn() -> Element, "First"), (cousins, "First")] {
        let body = body(&render(app));
        assert_eq!(body.matches(r#"tabindex="0""#).count(), 1, "{body}");
        let stop = body.find(r#"tabindex="0""#).unwrap();
        let second = body.find("Second").unwrap();
        assert!(
            stop < second,
            "the tab stop is not on the first row: {body}"
        );
        assert!(body[stop..second].contains(first), "{body}");
    }
}
