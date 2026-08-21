use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Code, Icon, Text, Tree, TreeLabel, TreeNode, TreeNodeRenderArgs, default_tree_render,
    },
    sx::sx,
};

use crate::icons::{FileIcon, FolderIcon};

#[derive(Clone, PartialEq)]
enum FileKind {
    Folder,
    Rust,
    Toml,
    Markdown,
}

#[derive(Clone, PartialEq)]
struct FileEntry {
    name: &'static str,
    kind: FileKind,
}

impl TreeLabel for FileEntry {
    fn tree_label(&self) -> String {
        self.name.to_string()
    }
}

fn file_tree() -> Vec<TreeNode<FileEntry>> {
    vec![
        TreeNode::new(
            "src",
            FileEntry {
                name: "src",
                kind: FileKind::Folder,
            },
        )
        .children(vec![
            TreeNode::new(
                "src/components",
                FileEntry {
                    name: "components",
                    kind: FileKind::Folder,
                },
            )
            .children(vec![
                TreeNode::new(
                    "src/components/list.rs",
                    FileEntry {
                        name: "list.rs",
                        kind: FileKind::Rust,
                    },
                ),
                TreeNode::new(
                    "src/components/tree.rs",
                    FileEntry {
                        name: "tree.rs",
                        kind: FileKind::Rust,
                    },
                ),
            ]),
            TreeNode::new(
                "src/hooks",
                FileEntry {
                    name: "hooks",
                    kind: FileKind::Folder,
                },
            ),
            TreeNode::new(
                "src/lib.rs",
                FileEntry {
                    name: "lib.rs",
                    kind: FileKind::Rust,
                },
            ),
        ]),
        TreeNode::new(
            "Cargo.toml",
            FileEntry {
                name: "Cargo.toml",
                kind: FileKind::Toml,
            },
        ),
        TreeNode::new(
            "README.md",
            FileEntry {
                name: "README.md",
                kind: FileKind::Markdown,
            },
        ),
    ]
}

fn category_tree() -> Vec<TreeNode<String>> {
    vec![
        TreeNode::new("a11y", "A11y".to_string()).children(vec![
            TreeNode::new("focus-trap", "Focus Trap".to_string()),
            TreeNode::new("visually-hidden", "Visually Hidden".to_string()),
        ]),
        TreeNode::new("data-display", "Data Display".to_string()).children(vec![
            TreeNode::new("icon", "Icon".to_string()),
            TreeNode::new("list", "List".to_string()),
            TreeNode::new("tree", "Tree".to_string()),
        ]),
    ]
}

#[component]
pub fn TreePage() -> Element {
    let mut basic_selected = use_signal(|| None::<String>);
    let mut files_selected = use_signal(|| None::<String>);

    rsx! {
        DocPage {
            title: "Tree",
            lead: rsx! {
                Text {
                    "Data-driven, not composed via children - pass "
                    Code { source: "Vec<TreeNode<T>>" }
                    " where "
                    Code { source: "T" }
                    " is entirely your own data shape. "
                    Code { source: "T" }
                    " only needs to implement "
                    Code { source: "TreeLabel" }
                    " (one method, "
                    Code { source: "fn tree_label(&self) -> String" }
                    "), which is used for keyboard typeahead and as the default row "
                    "rendering. A "
                    Code { source: "String" }
                    " already implements it, so a plain tree needs nothing further."
                }
                Text {
                    "Which nodes are expanded is "
                    Code { source: "Tree" }
                    "'s own business, not the caller's - "
                    Code { source: "default_expanded" }
                    " only seeds the initial state. Selection and what happens on click, "
                    "though, are entirely "
                    Code { source: "render_node" }
                    "'s call: "
                    Code { source: "Tree" }
                    " doesn't assume clicking a leaf means \"select it\" - a leaf might just "
                    "as well be a real link (see the sidebar on this page for that case)."
                }
                Text {
                    "Fully keyboard-navigable: arrow keys move between visible rows, "
                    Code { source: "Left" }
                    "/"
                    Code { source: "Right" }
                    " collapse/expand (or jump to the parent/first child), "
                    Code { source: "Home" }
                    "/"
                    Code { source: "End" }
                    " jump to the first/last row, and typing a letter jumps to the next match."
                }
            },
            DocSection {
                title: "Basic",
                Text {
                    "Plain "
                    Code { source: "String" }
                    " data. Branches use "
                    Code { source: "Tree" }
                    "'s own default rendering; leaves add a click handler that sets "
                    Code { source: "basic_selected" }
                    " - selection is this page's own state, not "
                    Code { source: "Tree" }
                    "'s."
                }
                Tree {
                    aria_label: "Component categories",
                    data: category_tree(),
                    render_node: move |args: TreeNodeRenderArgs<String>| {
                        if args.expanded.is_some() {
                            return default_tree_render(args);
                        }
                        rsx! {
                            Box {
                                component: "button",
                                r#type: "button",
                                // A real `<button>`, not a `<div onclick>` -
                                // gets real keyboard/click semantics for
                                // free, and `Tree`'s own Enter/Space
                                // handling looks for exactly this (an `a`
                                // or a `button`) to trigger.
                                sx: sx()
                                    .border("none")
                                    .background("none")
                                    .padding("6px 0")
                                    .color("inherit")
                                    .cursor("pointer"),
                                tabindex: args.tabindex,
                                onclick: move |_| basic_selected.set(Some(args.id.clone())),
                                "{args.data}"
                            }
                        }
                    },
                }
            }

            DocSection {
                title: "Custom rendering",
                Text {
                    "A "
                    Code { source: "render_node" }
                    " closure gets the node's own data plus its live "
                    Code { source: "expanded" }
                    " state, so content - like which folder icon to show - can react to it."
                }
                Tree {
                    aria_label: "Project files",
                    data: file_tree(),
                    render_node: move |args: TreeNodeRenderArgs<FileEntry>| {
                        let icon = if args.data.kind == FileKind::Folder {
                            rsx! {
                                Icon { variant: "transparent", size: "sm", color: "primary", FolderIcon {} }
                            }
                        } else {
                            rsx! {
                                Icon { variant: "transparent", size: "sm", color: "grey.6", FileIcon {} }
                            }
                        };
                        let id = args.id.clone();
                        rsx! {
                            Box {
                                component: "button",
                                r#type: "button",
                                sx: sx()
                                    .display("flex")
                                    .align_items("center")
                                    .gap("6px")
                                    .border("none")
                                    .background("none")
                                    .padding("6px 0")
                                    .color("inherit")
                                    .cursor("pointer"),
                                tabindex: args.tabindex,
                                onclick: move |_| files_selected.set(Some(id.clone())),
                                {icon}
                                "{args.data.name}"
                            }
                        }
                    },
                }
                if let Some(selected) = files_selected() {
                    Text { "Selected: " Code { source: "{selected}" } }
                }
            }
        }
    }
}
