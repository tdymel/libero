use pictogram_icons_lucide as lucide;
use std::collections::HashSet;

use crate::components::{
    Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{
        Button, Code, Flex, SvgData, Text, Tree, TreeItem, TreeLabel, TreeNode, TreeNodeRenderArgs,
        TreePart,
    },
    sx::sx,
    use_theme,
};

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

fn file_icon(entry: &FileEntry, expanded: Option<bool>) -> SvgData {
    match (&entry.kind, expanded) {
        (_, Some(true)) => lucide::folder_open::outlined,
        (FileKind::Folder, _) => lucide::folder::outlined,
        _ => lucide::file::outlined,
    }
}

/// The data, the controlled expansion and the row renderer are the demo's
/// fixture, not controls, but the code block has to print them.
const FIXED: [&str; 6] = [
    r#"aria_label: "Project files""#,
    "data: file_tree()",
    "expanded: expanded()",
    "onexpandedchange: move |next| expanded.set(next)",
    "current: selected()",
    r#"render_node: move |args: TreeNodeRenderArgs<FileEntry>| {
        let id = args.id.clone();
        rsx! {
            TreeItem {
                onclick: move |_| selected.set(Some(id.clone())),
                icon: file_icon(&args.data, args.expanded),
                "{args.data.name}"
            }
        }
    }"#,
];

/// The expansion and the selection are the page's state, not `Tree`'s, so the
/// buttons and the readout are part of the example.
fn wrap_selection(_: &DemoValues, code: &str) -> String {
    format!(
        "Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\").max_width(\"320px\"),\n{EXPAND_BUTTONS}{}    if let Some(selected) = selected() {{\n        Text {{ \"Selected: \" Code {{ source: \"{{selected}}\" }} }}\n    }}\n}}",
        indent(code)
    )
}

// snippet: ignore - a fragment of the Demo's code block, compiled there
const EXPAND_BUTTONS: &str = r#"    Flex { direction: "row", gap: "xs",
        Button { size: "xs", onclick: move |_| expanded.set(folders()), "Expand all" }
        Button { size: "xs", onclick: move |_| expanded.set(HashSet::new()), "Collapse all" }
    }
"#;

/// Every branch of `file_tree()`.
fn folders() -> HashSet<String> {
    HashSet::from(["src", "src/components"].map(str::to_string))
}

#[component]
pub fn TreePage() -> Element {
    let theme = use_theme();
    let mut selected = use_signal(|| None::<String>);
    let mut expanded = use_signal(folders);

    rsx! {
        DocPage {
            title: "Tree",
            source: "libero/src/components/navigation/tree",
            markdown: "/md/tree.md",
            properties: vec![
                props("Tree", vec![
                    prop("size", "Size")
                        .default("md")
                        .doc("Row gap and per-level indent together, from `List`'s scale. Set them apart through `sx`."),
                    prop("guides", "bool")
                        .default("false")
                        .doc("Draws a line down each open branch, under its chevron, and marks the `current` row's part of it. Theme default `TreeDefaults::guides`."),
                    prop("aria_label", "String").default("required").doc("The tree's accessible name."),
                    prop("data", "Vec<TreeNode<T>>").default("required").doc("The nodes, over your own data type."),
                    prop("render_node", "Callback<TreeNodeRenderArgs<T>, Element>")
                        .default("default_tree_render")
                        .doc("Each visible row's content. A custom one can still call the default for some rows."),
                    prop("default_expanded", "HashSet<String>").doc("The ids expanded at first. Read once, and ignored when `expanded` is set."),
                    prop("expanded", "HashSet<String>").doc(
                        "The expanded ids, controlled. The tree follows it and asks for every change through `onexpandedchange`. Unset, the tree keeps its own.",
                    ),
                    prop("current", "String").doc(
                        "The id of the node the user is on, such as a nav's current page. Tab into the tree lands on it, or on its collapsed branch. Its row carries `aria-current`.",
                    ),
                    prop("onexpandedchange", "EventHandler<HashSet<String>>")
                        .doc("Called with the whole new set of expanded ids. Store it when `expanded` is set; without `expanded` it only notifies."),
                    prop("parts", "Parts<TreePart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .parts("TreePart", vec![
                    (TreePart::Row, "A `treeitem`: the row and its open subtree. A `Tree` nested in a row's content matches too."),
                    (TreePart::Content, "The clickable line around `render_node`'s content."),
                    (TreePart::Group, "An open branch's list of children."),
                ]),
                props("TreeNode<T>", vec![
                    prop("id", "String").default("required").doc("The node's unique id."),
                    prop("data", "T").default("required").doc("Your data for this node."),
                    prop("children", "Vec<TreeNode<T>>").doc("Nested nodes. Without any, the node is a leaf."),
                    prop("disabled", "bool").default("false").doc("The arrow keys still reach it, but it does not activate, expand or collapse. Its descendants are disabled too."),
                ]).without_base_props(),
                props("TreeNodeRenderArgs<T>", vec![
                    prop("id", "String").doc("The node's id."),
                    prop("data", "T").doc("The node's data."),
                    prop("expanded", "Option<bool>").doc("`None` for a leaf."),
                    prop("disabled", "bool").doc("Whether the node or an ancestor is disabled."),
                    prop("tabindex", "&'static str")
                        .doc("Put it on any link or button in the row, or it adds a tab stop the arrow keys never reach."),
                    prop("depth", "usize").doc("0 at the top level, for custom indentation."),
                ]).without_base_props(),
                props("TreeItemContent", vec![
                    prop("icon", "SvgData").doc("Drawn after the chevron, before the label."),
                    prop("trailing_icon", "SvgData").doc("Drawn at the row's end."),
                    prop("trailing", "Element").doc("After `trailing_icon`, such as a count. Never interactive inside a `TreeItem`."),
                    prop("children", "Element").default("required").doc("The label."),
                ]),
                props("TreeItem", vec![
                    prop("onclick", "EventHandler<MouseEvent>").doc("Click handler."),
                    prop("icon", "SvgData").doc("Drawn after the chevron, before the label."),
                    prop("trailing_icon", "SvgData").doc("Drawn at the row's end."),
                    prop("trailing", "Element").doc("After `trailing_icon`, such as a count. Never interactive: it sits in the button."),
                    prop("children", "Element").default("required").doc("The label."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters the tree, one tab stop.")
                .key(["Up", "Down"], "Moves between visible rows.")
                .key(["Left", "Right"], "Collapses and expands, or jumps to the parent and first child.")
                .key(["Home", "End"], "Jumps to the first or last row.")
                .key(["Enter", "Space"], "Opens or closes a branch, or clicks a leaf's link or button.")
                .key(["*"], "Opens every closed sibling of the row.")
                .handles([
                    "Typing jumps to the next row whose `tree_label` matches.",
                    "A disabled node is still reachable, but nothing activates, expands or collapses it.",
                    "`TreeItem` takes `args.tabindex` for you.",
                ])
                .must([
                    "Name the tree with `aria_label`. It is required.",
                    "Pass `args.tabindex` to any link or button `render_node` draws, or it adds a tab stop the arrow keys never reach.",
                ]),
            lead: rsx! {
                Text {
                    "A tree view over "
                    Code { source: "Vec<TreeNode<T>>" }
                    ", where "
                    Code { source: "T" }
                    " is your own data type. "
                    Code { source: "T" }
                    " implements "
                    Code { source: "TreeLabel" }
                    ", whose text feeds typeahead and the default row. "
                    Code { source: "String" }
                    " already does. The tree keeps its own expanded state, seeded by "
                    Code { source: "default_expanded" }
                    ", or follows a controlled "
                    Code { source: "expanded" }
                    " paired with "
                    Code { source: "onexpandedchange" }
                    ", as here."
                }
                Text {
                    Code { source: "Tree" }
                    " has no selection. "
                    Code { source: "render_node" }
                    " decides what a row does. "
                    Code { source: "TreeItemContent" }
                    " is the standard row: the chevron, an "
                    Code { source: "icon" }
                    ", the label and a "
                    Code { source: "trailing_icon" }
                    ". The default row is one. "
                    Code { source: "TreeItem" }
                    " lays a row out the same way as a button that picks up the tab stop and "
                    Code { source: "disabled" }
                    ". A row that links somewhere is a "
                    Code { source: "NavLink" }
                    " with "
                    Code { source: "args.tabindex" }
                    ", as in this site's sidebar."
                }
                Text {
                    Code { source: "guides" }
                    " draws a line down each open branch and marks the "
                    Code { source: "current" }
                    " row's part of it, as this site's sidebar does. Pick a file below to see it."
                }
            },
            // snippet: item #[derive(Clone, PartialEq)] struct FileEntry { name: &'static str }
            // snippet: item impl TreeLabel for FileEntry { fn tree_label(&self) -> String { self.name.to_string() } }
            // snippet: item fn file_tree() -> Vec<TreeNode<FileEntry>> { Vec::new() }
            // snippet: item fn file_icon(_: &FileEntry, _: Option<bool>) -> libero::components::SvgData { pictogram_icons_lucide::file::outlined }
            // snippet: item use std::collections::HashSet;
            // snippet: item fn folders() -> HashSet<String> { HashSet::new() }
            // snippet: let mut selected = use_signal(|| None::<String>);
            // snippet: let mut expanded = use_signal(folders);
            Demo {
                component: "Tree",
                children_text: "",
                fixed: FIXED.map(str::to_string).to_vec(),
                controls: vec![
                    Control::switch("guides"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.tree.size.as_str()),
                ],
                render: move |values: DemoValues| rsx! {
                    Flex {
                        direction: "column",
                        gap: "sm",
                        // Pinned, so the selection readout does not reflow the
                        // preview under the pointer.
                        sx: sx().width("100%").max_width("320px"),
                        Flex { direction: "row", gap: "xs",
                            Button { size: "xs", onclick: move |_| expanded.set(folders()), "Expand all" }
                            Button { size: "xs", onclick: move |_| expanded.set(HashSet::new()), "Collapse all" }
                        }
                        Tree {
                            aria_label: "Project files",
                            data: file_tree(),
                            expanded: expanded(),
                            onexpandedchange: move |next| expanded.set(next),
                            current: selected(),
                            guides: values.str("guides") == "true",
                            size: or_unset(values.str("size")),
                            render_node: move |args: TreeNodeRenderArgs<FileEntry>| {
                                let id = args.id.clone();
                                rsx! {
                                    TreeItem {
                                        onclick: move |_| selected.set(Some(id.clone())),
                                        icon: file_icon(&args.data, args.expanded),
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
