use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Icon, Text, Title, Tree, TreeLabel, TreeNode, TreeNodeRenderArgs},
    hooks::use_tree_state,
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
    let basic = use_tree_state();
    let files = use_tree_state();

    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Tree" }
                Text {
                    "Data-driven, not composed via children - pass "
                    Code { "Vec<TreeNode<T>>" }
                    " where "
                    Code { "T" }
                    " is entirely your own data shape. "
                    Code { "T" }
                    " only needs to implement "
                    Code { "TreeLabel" }
                    " (one method, "
                    Code { "fn tree_label(&self) -> String" }
                    "), which is used for keyboard typeahead and as the default row "
                    "rendering. A "
                    Code { "String" }
                    " already implements it, so a plain tree needs nothing further."
                }
                Text {
                    "Fully keyboard-navigable: arrow keys move between visible rows, "
                    Code { "Left" }
                    "/"
                    Code { "Right" }
                    " collapse/expand (or jump to the parent/first child), "
                    Code { "Home" }
                    "/"
                    Code { "End" }
                    " jump to the first/last row, and typing a letter jumps to the next "
                    "match. "
                    Code { "Tree" }
                    " is fully controlled - "
                    Code { "use_tree_state()" }
                    " is a small convenience wrapper for the common case, wiring up the "
                    Code { "expanded" }
                    "/"
                    Code { "selected" }
                    " state for you."
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Basic" }
                Text { "Plain " Code { "String" } " data - no " Code { "render_node" } " needed." }
                Tree {
                    aria_label: "Component categories",
                    data: category_tree(),
                    expanded: basic.expanded(),
                    onexpandedchange: move |expanded| basic.set_expanded(expanded),
                    selected: basic.selected(),
                    onselectedchange: move |selected| basic.set_selected(selected),
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Custom rendering" }
                Text {
                    "A "
                    Code { "render_node" }
                    " closure gets the node's own data plus its live "
                    Code { "expanded" }
                    "/"
                    Code { "selected" }
                    " state, so content - like which folder icon to show - can react to it."
                }
                Tree {
                    aria_label: "Project files",
                    data: file_tree(),
                    expanded: files.expanded(),
                    onexpandedchange: move |expanded| files.set_expanded(expanded),
                    selected: files.selected(),
                    onselectedchange: move |selected| files.set_selected(selected),
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
                        rsx! {
                            {icon}
                            "{args.data.name}"
                        }
                    },
                }
                if let Some(selected) = files.selected() {
                    Text { "Selected: " Code { "{selected}" } }
                }
            }
        }
    }
}
