use std::collections::HashSet;

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

/// No selection model: every row says `aria-selected="false"`, so Chrome does
/// not announce the tab stop as selected; `current` is `aria-current` instead.
#[test]
fn rows_are_unselected_and_the_current_row_is_aria_current() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string()),
            TreeNode::new("b", "Beta".to_string())
                .children(vec![TreeNode::new("c", "Gamma".to_string())]),
        ];

        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data,
                    default_expanded: ["b".to_string()].into(),
                    current: "c",
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let rows: Vec<&str> = body.split("<li").skip(1).collect();

    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| {
        let tag = &row[..row.find('>').unwrap()];
        tag.contains(r#"aria-selected="false""#)
    }));
    let current: Vec<&&str> = rows
        .iter()
        .filter(|row| row[..row.find('>').unwrap()].contains("aria-current"))
        .collect();
    assert_eq!(current.len(), 1);
    assert!(current[0].contains(r#"aria-current="true""#));
    assert!(current[0].contains("Gamma"));
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

/// Todo 430: a child of a disabled branch is disabled too, for the keys and
/// `render_node` as for the mouse the branch's `pointer-events` already stops.
#[test]
fn a_disabled_branch_disables_its_children() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string())
                .disabled(true)
                .children(vec![TreeNode::new("a1", "Child".to_string())]),
            TreeNode::new("b", "Beta".to_string()),
        ];
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data,
                    default_expanded: ["a".to_string()].into(),
                    render_node: move |args: TreeNodeRenderArgs<String>| rsx! {
                        span { "data-probe": "{args.id}={args.disabled}" }
                    },
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(body.contains(r#"data-probe="a1=true""#), "{body}");
    assert!(body.contains(r#"data-probe="b=false""#), "{body}");
    assert_eq!(body.matches(r#"aria-disabled="true""#).count(), 2, "{body}");
}

/// Todo 770: a controlled `expanded` wins over `default_expanded`.
#[test]
fn a_controlled_expanded_opens_its_branches_only() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string())
                .children(vec![TreeNode::new("a1", "Child A".to_string())]),
            TreeNode::new("b", "Beta".to_string())
                .children(vec![TreeNode::new("b1", "Child B".to_string())]),
        ];
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data,
                    default_expanded: ["a".to_string()].into(),
                    expanded: HashSet::from(["b".to_string()]),
                    onexpandedchange: |_| {},
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(body.contains("Child B"), "{body}");
    assert!(!body.contains("Child A"), "{body}");
}
