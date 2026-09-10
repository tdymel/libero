use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, or_unset, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, Icon, Text, Tree, TreeItem, TreeLabel, TreeNode, TreeNodeRenderArgs},
    sx::sx,
    use_theme,
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

fn file_icon(entry: &FileEntry) -> Element {
    let folder = entry.kind == FileKind::Folder;
    rsx! {
        Icon {
            variant: "standard",
            size: "sm",
            color: if folder { "primary" } else { "muted.6" },
            if folder { FolderIcon {} } else { FileIcon {} }
        }
    }
}

/// The data, the seeded expansion and the row renderer are the demo's
/// fixture, not props a control varies - but the code block has to print them,
/// so they are `fixed`.
const FIXED: [&str; 4] = [
    r#"aria_label: "Project files""#,
    "data: file_tree()",
    r#"default_expanded: ["src", "src/components"].map(str::to_string).into()"#,
    r#"render_node: move |args: TreeNodeRenderArgs<FileEntry>| {
        let id = args.id.clone();
        rsx! {
            TreeItem {
                onclick: move |_| selected.set(Some(id.clone())),
                {file_icon(&args.data)}
                "{args.data.name}"
            }
        }
    }"#,
];

/// Selection is the page's state, not `Tree`'s, so the readout is part of the
/// example rather than something the component provides.
fn wrap_selection(_: &DemoValues, code: &str) -> String {
    format!(
        "Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\").max_width(\"320px\"),\n{}    if let Some(selected) = selected() {{\n        Text {{ \"Selected: \" Code {{ source: \"{{selected}}\" }} }}\n    }}\n}}",
        indent(code)
    )
}

#[component]
pub fn TreePage() -> Element {
    let theme = use_theme();
    let mut selected = use_signal(|| None::<String>);

    rsx! {
        DocPage {
            title: "Tree",
            source: "libero/src/components/navigation/tree",
            markdown: "/md/tree.md",
            properties: vec![
                props("Tree", vec![
                    prop("size", "Size")
                        .default("md")
                        .doc("Row gap and per-level indent together - `List`'s scale, since `Tree` renders through it. Off-scale, or the two apart, goes through `sx`."),
                    prop("aria_label", "String").doc("Required by WAI-ARIA's tree pattern."),
                    prop("data", "Vec<TreeNode<T>>").doc("The tree's data, entirely your own shape."),
                    prop("render_node", "Callback<TreeNodeRenderArgs<T>, Element>")
                        .default("default_tree_render")
                        .doc("Each visible row's content. The default can also be called selectively - say, default branches and `NavLink` leaves."),
                    prop("default_expanded", "HashSet<String>").doc("Seeds `Tree`'s internal state once. Not a controlled prop."),
                    prop("current", "Option<String>").doc(
                        "The id of the node where the user is - a nav's current page. Tab into the tree lands on it rather than on the first row, until the arrow keys move on; when it changes, the tab stop follows it. Inside a collapsed branch, the tab stop goes to the branch.",
                    ),
                    prop("onexpandedchange", "EventHandler<HashSet<String>>")
                        .doc("Notification only - it doesn't drive rendering."),
                ]),
                props("TreeNode<T>", vec![
                    prop("id", "String").doc("The node's identity, used for expansion state and keyboard navigation."),
                    prop("data", "T").doc("The caller's own data for this node."),
                    prop("children", "Vec<TreeNode<T>>").doc("Nested nodes - an empty vec makes this a leaf."),
                    prop("disabled", "bool").default("false").doc("Reachable by the arrow keys; nothing activates, expands or collapses it."),
                ]).without_base_props(),
                props("TreeNodeRenderArgs<T>", vec![
                    prop("id", "String").doc("The node's id."),
                    prop("data", "T").doc("The node's data."),
                    prop("expanded", "bool").doc("`None` for a leaf - no children, no chevron, no `aria-expanded`."),
                    prop("disabled", "bool").doc("Whether the node is disabled."),
                    prop("tabindex", "&'static str")
                        .doc("Apply to any interactive element your content renders - the row is already the roving tab stop, without this a link/button adds a second one arrow keys never move."),
                    prop("depth", "usize").doc("0 at top level. Lets `render_node` take indentation over entirely."),
                ]).without_base_props(),
                props("TreeItem", vec![
                    prop("onclick", "EventHandler<MouseEvent>").doc("Fires on click."),
                    prop("children", "Element").doc("The row's content - an icon, the label."),
                ]),
            ],
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
                    " already implements it, so a plain tree needs nothing further - and "
                    Code { source: "default_tree_render(args)" }
                    " gives one row that rendering back."
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
                    "as well be a real link (see the sidebar on this page for that case). The "
                    "closure also gets the row's live "
                    Code { source: "expanded" }
                    " state, so content can react to it."
                }
                Text {
                    Code { source: "TreeItem" }
                    " is the row content a clickable leaf wants: the "
                    Code { source: "button" }
                    " Enter/Space activation looks for, with its chrome stripped and the "
                    "page's type inherited. It takes the row's tab stop and "
                    Code { source: "disabled" }
                    " from the row itself, so neither can be forgotten. A row that is a real "
                    "link is a "
                    Code { source: "NavLink" }
                    " instead - pass it "
                    Code { source: "args.tabindex" }
                    " yourself."
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
                Text {
                    Code { source: "Tree" }
                    " renders through "
                    Code { source: "List" }
                    " and has no size scale of its own: "
                    Code { source: "size" }
                    " is the row gap and the per-level indent together. Off-scale, or the "
                    "two apart, goes through "
                    Code { source: "sx" }
                    " - rows are yours, so their type scale is too."
                }
            },
            // snippet: item #[derive(Clone, PartialEq)] struct FileEntry { name: &'static str }
            // snippet: item impl TreeLabel for FileEntry { fn tree_label(&self) -> String { self.name.to_string() } }
            // snippet: item fn file_tree() -> Vec<TreeNode<FileEntry>> { Vec::new() }
            // snippet: item fn file_icon(_: &FileEntry) -> Element { rsx! {} }
            // snippet: let mut selected = use_signal(|| None::<String>);
            Demo {
                component: "Tree",
                children_text: "",
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.tree.size.as_str()),
                ],
                render: move |values: DemoValues| rsx! {
                    Flex {
                        direction: "column",
                        gap: "sm",
                        // Pinned: the selection readout appearing must not
                        // reflow the preview under the pointer.
                        sx: sx().width("100%").max_width("320px"),
                        Tree {
                            aria_label: "Project files",
                            data: file_tree(),
                            default_expanded: ["src", "src/components"]
                                .map(str::to_string)
                                .into(),
                            size: or_unset(values.str("size")),
                            render_node: move |args: TreeNodeRenderArgs<FileEntry>| {
                                let id = args.id.clone();
                                rsx! {
                                    TreeItem {
                                        onclick: move |_| selected.set(Some(id.clone())),
                                        {file_icon(&args.data)}
                                        "{args.data.name}"
                                    }
                                }
                            },
                        }
                        if let Some(selected) = selected() {
                            Text { "Selected: " Code { source: "{selected}" } }
                        }
                    }
                },
                wrap: Wrap(wrap_selection),
            }
        }
    }
}
