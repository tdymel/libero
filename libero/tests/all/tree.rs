use std::collections::HashSet;

use crate::common::{attributes_of, body, render, tag_with, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{NavLink, Tree, TreeItem, TreeItemContent, TreeNode, TreeNodeRenderArgs},
};

/// Outside a row a `TreeItem` is a tab-stop button; inside one it is the treeitem's
/// own label (D41-F1), so the name is read once.
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

    /// The control: outside a tree it is an ordinary button and a tab stop.
    fn loose() -> Element {
        rsx! { LiberoProvider { TreeItem { "Alpha" } } }
    }

    // Inside a row the treeitem is the widget: a label, not a second tab stop or button.
    let in_row = body(&render(app));
    assert!(!in_row.contains("<button"), "{in_row}");
    let label = tag_with(&in_row, "data-tree-activate");
    assert_eq!(label.get("tabindex"), None, "{label:?}");
    let alone = attributes_of(&body(&render(loose)), "button");
    assert_eq!(
        alone.get("tabindex").map(String::as_str),
        Some("0"),
        "{alone:?}"
    );
}

/// Todo 3785: the chevron's turn takes the theme's duration and stops under reduced motion.
#[test]
fn the_chevron_turn_takes_the_theme_duration_and_a_reduced_motion_guard() {
    fn app() -> Element {
        rsx! { LiberoProvider { TreeItem { "Alpha" } } }
    }
    let html = render(app);

    assert!(
        html.contains("transition:transform var(--lsx-tree-chevron-duration) ease;"),
        "{html}"
    );
    assert!(
        html.contains("--lsx-tree-chevron-duration:120ms;"),
        "{html}"
    );
    assert!(
        html.split("@media (prefers-reduced-motion: reduce)")
            .skip(1)
            .any(|block| block.contains("[data-tree-chevron]{transition:none;}")),
        "{html}"
    );
}

/// Read at the first render: a closed branch says `false`, an open one `true`
/// and shows its group, a leaf says nothing.
#[test]
fn a_branch_says_whether_it_is_open_from_the_first_render() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("closed", "Closed".to_string())
                .children(vec![TreeNode::new("c1", "Hidden".to_string())]),
            TreeNode::new("open", "Open".to_string())
                .children(vec![TreeNode::new("o1", "Shown".to_string())]),
            TreeNode::new("leaf", "Leaf".to_string()),
        ];
        rsx! {
            LiberoProvider {
                Tree { aria_label: "Files", data, default_expanded: HashSet::from(["open".to_string()]) }
            }
        }
    }
    let html = body(&render(app));

    let row = |id: &str| tag_with(&html, &format!(r#"data-tree-id="{id}""#));
    assert_eq!(row("closed")["aria-expanded"], "false");
    assert_eq!(row("open")["aria-expanded"], "true");
    assert!(!row("leaf").contains_key("aria-expanded"));
    assert!(html.contains("Shown"), "{html}");
    assert!(!html.contains("Hidden"), "{html}");
    assert_eq!(tags_with(&html, r#"role="group""#).len(), 1, "{html}");
}

#[test]
fn an_empty_tree_draws_no_rows() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Tree::<String> { aria_label: "Files", data: vec![] } }
        }
    }
    let html = body(&render(app));

    assert_eq!(tag_with(&html, r#"role="tree""#)["aria-label"], "Files");
    assert!(tags_with(&html, r#"role="treeitem""#).is_empty(), "{html}");
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
    // Drawn too without guides: NavLink's tint and start bar (todo 1571).
    let content = tag_with(current[0], r#"data-slot="content""#);
    assert!(
        content
            .get("data-state")
            .is_some_and(|state| state.split(' ').any(|state| state == "current")),
        "{content:?}"
    );
}

/// Todo 2037: an active `NavLink` in the current row leaves the look to the row, so the
/// tint and start bar are drawn once; elsewhere it keeps its own.
#[test]
fn an_active_nav_link_in_the_current_row_leaves_the_look_to_the_row() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("a", "Alpha".to_string()),
            TreeNode::new("b", "Beta".to_string()),
        ];
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Pages",
                    data,
                    current: "b",
                    render_node: move |args: TreeNodeRenderArgs<String>| rsx! {
                        NavLink { to: "/page", active: true, "{args.data}" }
                    },
                }
            }
        }
    }
    let html = body(&render(app));

    let links = tags_with(&html, r#"aria-current="page""#);
    assert_eq!(links.len(), 2, "{html}");
    let active = |link: &std::collections::BTreeMap<String, String>| {
        link.get("data-state")
            .is_some_and(|state| state.split(' ').any(|state| state == "active"))
    };
    assert!(
        active(&links[0]),
        "a plain row keeps the link's look: {links:?}"
    );
    assert!(!active(&links[1]), "the current row draws it: {links:?}");
}

/// Todo 3837: a `NavLink` in a disabled row, or under a disabled branch, goes nowhere; an
/// explicit `disabled: false` keeps its link.
#[test]
fn a_nav_link_in_a_disabled_row_has_no_href() {
    fn app() -> Element {
        let data = vec![
            TreeNode::new("open", "Open".to_string()),
            TreeNode::new("off", "Off".to_string()).disabled(true),
            TreeNode::new("branch", "Branch".to_string())
                .disabled(true)
                .children(vec![TreeNode::new("child", "Child".to_string())]),
            TreeNode::new("kept", "Kept".to_string()).disabled(true),
        ];
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Pages",
                    data,
                    default_expanded: ["branch".to_string()].into(),
                    render_node: move |args: TreeNodeRenderArgs<String>| rsx! {
                        NavLink {
                            to: "/page",
                            disabled: (args.id == "kept").then_some(false),
                            "{args.data}"
                        }
                    },
                }
            }
        }
    }
    let html = body(&render(app));

    let link = |label: &str| {
        let at = html.find(&format!(">{label}</a>")).expect(label);
        let tag = &html[html[..at].rfind("<a").unwrap()..=at];
        attributes_of(tag, "a")
    };
    for label in ["Off", "Branch", "Child"] {
        let link = link(label);
        assert!(!link.contains_key("href"), "{label}: {link:?}");
        assert_eq!(link["aria-disabled"], "true", "{label}: {link:?}");
    }
    for label in ["Open", "Kept"] {
        let link = link(label);
        assert!(link.contains_key("href"), "{label}: {link:?}");
        assert!(!link.contains_key("aria-disabled"), "{label}: {link:?}");
    }
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

fn nested() -> Vec<TreeNode<String>> {
    vec![
        TreeNode::new("a", "Alpha".to_string())
            .children(vec![TreeNode::new("a1", "Child".to_string())]),
        TreeNode::new("b", "Beta".to_string()),
    ]
}

fn guided(guides: bool, current: &'static str) -> Element {
    rsx! {
        LiberoProvider {
            Tree {
                aria_label: "Files",
                data: nested(),
                default_expanded: ["a".to_string()].into(),
                guides,
                current,
            }
        }
    }
}

/// Todo 1135: `guides` marks every row, and only a nested `current` row's
/// content carries the active segment.
#[test]
fn guides_mark_the_rows_and_a_nested_current_row() {
    let guided = |app: fn() -> Element| body(&render(app)).to_string();

    let on = guided(|| self::guided(true, "a1"));
    assert_eq!(on.matches("<li").count(), 3, "{on}");
    assert_eq!(on.matches(r#"guides"#).count(), 3, "{on}");
    assert_eq!(on.matches("guide-current").count(), 1, "{on}");
    let marked = &on[on.find("guide-current").unwrap()..];
    assert!(
        marked[..marked.find("</li>").unwrap()].contains("Child"),
        "{on}"
    );

    let top = guided(|| self::guided(true, "b"));
    assert!(
        !top.contains("guide-current"),
        "a top-level row has no guide: {top}"
    );

    let off = guided(|| self::guided(false, "a1"));
    assert!(!off.contains("guides"), "{off}");
}

/// Todo 1136: the default row is a `TreeItemContent`: chevron, then the label.
#[test]
fn the_default_row_is_the_standard_item() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tree { aria_label: "Files", data: nested() }
            }
        }
    }

    let body = body(&render(app));
    assert_eq!(body.matches("data-tree-chevron").count(), 1, "{body}");
    assert_eq!(body.matches("data-tree-label").count(), 2, "{body}");
}

/// Todo 1136: icons go before and after the label; a `TreeItem` draws the same parts.
#[test]
fn a_standard_item_draws_its_icons_around_the_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tree {
                    aria_label: "Files",
                    data: nested(),
                    render_node: move |args: TreeNodeRenderArgs<String>| match args.expanded {
                        Some(_) => rsx! {
                            TreeItemContent {
                                icon: pictogram_icons_lucide::folder::outlined,
                                trailing: rsx! { span { "data-probe": "count" } },
                                "{args.data}"
                            }
                        },
                        None => rsx! {
                            TreeItem {
                                trailing_icon: pictogram_icons_lucide::star::outlined,
                                "{args.data}"
                            }
                        },
                    },
                }
            }
        }
    }

    let body = body(&render(app));
    assert_eq!(body.matches("data-tree-icon").count(), 2, "{body}");
    let alpha = &body[..body.find("Beta").unwrap()];
    let icon = alpha.find("data-tree-icon").unwrap();
    let label = alpha.find("Alpha").unwrap();
    let probe = alpha.find("data-probe").unwrap();
    assert!(icon < label && label < probe, "{alpha}");
    let beta = &body[body.find("data-tree-activate").unwrap()..];
    assert!(
        beta.find("Beta").unwrap() < beta.find("data-tree-icon").unwrap(),
        "{beta}"
    );
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::body;
    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Tree, TreeNode},
    };

    /// A toggle keeps a branch row's content element, so the listener the first
    /// render registered still toggles it. It used to swap templates, and the
    /// second click landed on whatever took over the freed id.
    #[test]
    fn a_tree_branch_row_toggles_on_every_click() {
        fn app() -> Element {
            let node = |id: &str| TreeNode::new(id, id.to_string());
            rsx! {
                LiberoProvider {
                    for _ in 0..3 {
                        Tree { aria_label: "t", data: vec![node("a").children(vec![node("a1"), node("a2")])] }
                    }
                }
            }
        }

        let Page { mut dom, rec: find } = Page::build(app);
        // A tree root's click listener only moves focus, so clicking it is a no-op.
        assert_eq!(find.clicks.len(), 6, "one per branch row and tree root");

        for expanded in ["true", "false", "true"] {
            for &row in &find.clicks {
                dom.runtime()
                    .handle_event("click", Event::new(click_event(), true), row);
            }
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            let html = body(&dioxus_ssr::render(&dom));
            assert_eq!(
                html.matches(&format!(r#"aria-expanded="{expanded}""#))
                    .count(),
                3,
                "{html}"
            );
        }
    }
}

#[test]
fn a_blank_name_is_not_put_on_the_tree() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tree { aria_label: " ", data: vec![TreeNode::new("a", "Alpha".to_string())] }
            }
        }
    }

    let tree = tag_with(&body(&render(app)), "role=\"tree\"");
    assert_eq!(tree.get("aria-label"), None, "{tree:?}");
}
