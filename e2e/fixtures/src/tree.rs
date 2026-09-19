//! `Tree`.

use std::collections::HashSet;

use dioxus::prelude::*;
use libero::components::{Flex, NavLink, Tree, TreeItem, TreeNode, TreeNodeRenderArgs};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tree", || rsx! { TreePage {} }),
    ("/tree/links", || rsx! { LinkTreePage {} }),
    ("/tree/links/arrived", || rsx! { "arrived" }),
    ("/tree/delete", || rsx! { DeleteTreePage {} }),
    ("/tree/default", || rsx! { DefaultTreePage {} }),
    ("/tree/controlled", || rsx! { ControlledTreePage {} }),
    ("/tree/chevron", || rsx! { ChevronTreePage {} }),
    ("/tree/activate", || rsx! { ActivateTreePage {} }),
];

/// One branch, one leaf: the branch's chevron turns as it expands.
#[component]
fn ChevronTreePage() -> Element {
    let data = vec![
        TreeNode::new("src", "src").children(vec![TreeNode::new("src/lib.rs", "lib.rs")]),
        TreeNode::new("README.md", "README.md"),
    ];
    rsx! {
        Tree { aria_label: "Files", data }
    }
}

/// Leaves that render a link whose click `#activated` records (todo 4).
#[component]
fn ActivateTreePage() -> Element {
    let mut activated = use_signal(|| String::from("none"));
    let data = vec![
        TreeNode::new("here", "Here"),
        TreeNode::new("elsewhere", "Elsewhere"),
    ];
    rsx! {
        output { id: "activated", "{activated}" }
        Tree {
            aria_label: "Pages",
            data,
            render_node: move |args: TreeNodeRenderArgs<&'static str>| {
                let id = args.id.to_string();
                rsx! {
                    a {
                        tabindex: args.tabindex,
                        onclick: move |event: MouseEvent| {
                            event.prevent_default();
                            activated.set(id.clone());
                        },
                        "{args.data}"
                    }
                }
            },
        }
    }
}

/// Todo 770: a controlled `expanded`. "o" opens `docs` from outside, as the docs
/// nav reveals the current page; the second tree records requests and keeps
/// its set, so a key there can only ask.
#[component]
fn ControlledTreePage() -> Element {
    let data = || {
        vec![
            TreeNode::new("src", "src").children(vec![TreeNode::new("src/lib.rs", "lib.rs")]),
            TreeNode::new("docs", "docs")
                .children(vec![TreeNode::new("docs/index.md", "index.md")]),
        ]
    };
    let mut open = use_signal(HashSet::<String>::new);
    let mut requests = use_signal(|| 0);

    rsx! {
        div {
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Character("o".into()) {
                    open.write().insert("docs".to_string());
                }
            },
            Tree {
                id: "stored",
                aria_label: "Stored",
                data: data(),
                expanded: open(),
                onexpandedchange: move |next| open.set(next),
            }
        }
        Tree {
            id: "fixed",
            aria_label: "Fixed",
            data: data(),
            expanded: HashSet::new(),
            onexpandedchange: move |_| requests += 1,
        }
        span { id: "requests", "data-requests": "{requests}" }
    }
}

/// The default row render, the one that draws the chevron.
#[component]
fn DefaultTreePage() -> Element {
    let data = vec![
        TreeNode::new("src", "src").children(vec![TreeNode::new("src/lib.rs", "lib.rs")]),
        TreeNode::new("README.md", "README.md"),
    ];

    rsx! {
        Tree { aria_label: "Project files", data }
    }
}

/// Delete removes the focused row from `data`, as a file manager would.
#[component]
fn DeleteTreePage() -> Element {
    let mut data = use_signal(|| {
        ["a.rs", "b.rs", "c.rs"]
            .map(|id| TreeNode::new(id, id))
            .to_vec()
    });

    rsx! {
        div {
            onkeydown: move |event: KeyboardEvent| {
                if event.key() != Key::Delete {
                    return;
                }
                let focused = document::eval("return document.activeElement?.dataset.treeId");
                spawn(async move {
                    if let Ok(id) = focused.await
                        && let Some(id) = id.as_str()
                    {
                        data.write().retain(|node| node.id != id);
                    }
                });
            },
            Tree {
                aria_label: "Files",
                data: data(),
                render_node: move |args: TreeNodeRenderArgs<&'static str>| {
                    rsx! {
                        TreeItem { tabindex: args.tabindex, "{args.data}" }
                    }
                },
            }
        }
    }
}

/// Leaves that are links, as the docs sidebar renders them: Enter on one has
/// to navigate (todo 4).
#[component]
fn LinkTreePage() -> Element {
    let data = vec![
        TreeNode::new("/tree/links", "Here"),
        TreeNode::new("/tree/links/arrived", "Elsewhere"),
    ];

    rsx! {
        Tree {
            aria_label: "Pages",
            data,
            render_node: move |args: TreeNodeRenderArgs<&'static str>| {
                rsx! {
                    NavLink { to: args.id, tabindex: args.tabindex, "{args.data}" }
                }
            },
        }
    }
}

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
