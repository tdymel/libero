use pictogram_icons_lucide as lucide;
use std::collections::HashSet;

use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, indent, or_unset, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{
        Button, Code, Flex, SvgData, Text, Tree, TreeItem, TreeLabel, TreeNode, TreeNodeRenderArgs,
        TreePart,
    },
    hooks::use_timeout,
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
            )
            .children(vec![TreeNode::new(
                "src/hooks/use_tree.rs",
                FileEntry {
                    name: "use_tree.rs",
                    kind: FileKind::Rust,
                },
            )]),
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

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("tree.rs"));

/// A branch whose children have not arrived holds a disabled "Loading…" row,
/// which keeps it a branch.
fn lazy_file_tree(loaded: &HashSet<String>) -> Vec<TreeNode<FileEntry>> {
    fn hold(node: TreeNode<FileEntry>, loaded: &HashSet<String>) -> TreeNode<FileEntry> {
        let children = match (node.children.is_empty(), loaded.contains(&node.id)) {
            (true, _) => Vec::new(),
            (false, true) => node.children.into_iter().map(|c| hold(c, loaded)).collect(),
            (false, false) => {
                let entry = FileEntry {
                    name: "Loading…",
                    kind: FileKind::Rust,
                };
                vec![TreeNode::new(format!("{}/loading", node.id), entry).disabled(true)]
            }
        };
        TreeNode::new(node.id, node.data).children(children)
    }
    file_tree()
        .into_iter()
        .map(|node| hold(node, loaded))
        .collect()
}

fn lazy(values: &DemoValues) -> bool {
    values.str("lazy_loading") == "true"
}

/// The tree's identity, the controlled `current` and the row renderer are the
/// demo's fixture, not controls, but the code block has to print them.
fn fixed() -> Vec<String> {
    let (wiring, render) = (FILE.section("wiring"), FILE.section("render"));
    let lines = wiring.lines().chain([render.as_str()]);
    lines
        .map(|line| line.trim_end_matches(',').to_string())
        .collect()
}

/// The expansion is the page's state, not `Tree`'s, so the buttons and the
/// readout are part of the example. Lazy loading swaps the buttons for its
/// fetch state.
fn wrap_selection(values: &DemoValues, code: &str) -> String {
    let (buttons, status) = match lazy(values) {
        true => (
            String::new(),
            format!("{}\n{}", FILE.section("status"), FILE.section("loading")),
        ),
        false => (FILE.section("buttons"), FILE.section("status")),
    };
    let flex = format!(
        "Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\").max_width(\"320px\"),\n{}{}    // Always mounted, so a screen reader hears the text it gains.\n{}}}",
        indent(&buttons),
        indent(code),
        indent(&status)
    );
    match lazy(values) {
        true => format!("{}\n\nrsx! {{\n{}}}", FILE.section("lazy"), indent(&flex)),
        false => flex,
    }
}

/// The data, the expansion and its handler, for either way of loading.
fn loading_code(_: &Control, values: &DemoValues) -> Vec<String> {
    match lazy(values) {
        true => vec![
            "data: lazy_file_tree(&loaded())".to_string(),
            "expanded: lazy_expanded()".to_string(),
            "onexpandedchange: on_lazy_expand".to_string(),
        ],
        false => vec![
            "data: file_tree()".to_string(),
            "expanded: expanded()".to_string(),
            "onexpandedchange: move |next| expanded.set(next)".to_string(),
        ],
    }
}

/// Every branch of `file_tree()`.
fn folders() -> HashSet<String> {
    HashSet::from(["src", "src/components"].map(str::to_string))
}

/// The selection props the code prints: `selected` with its handler, and `multiple` for the multi mode.
fn selection_code(_: &Control, values: &DemoValues) -> Vec<String> {
    let mut props = Vec::new();
    if values.str("selection") != "none" {
        props.push("selected: picked()".to_string());
        props.push("onselectedchange: move |next| picked.set(next)".to_string());
    }
    if values.str("selection") == "multiple" {
        props.push("multiple: true".to_string());
    }
    props
}

/// `checked` with its handler, printed only while the checkboxes are on.
fn checkbox_code(_: &Control, values: &DemoValues) -> Vec<String> {
    match values.str("checkboxes") == "true" {
        true => vec![
            "checked: checks()".to_string(),
            "oncheckedchange: move |next| checks.set(next)".to_string(),
        ],
        false => vec![],
    }
}

/// Mounted while lazy loading is on; switching it off forgets what was fetched.
#[component]
fn LazyReset(
    loaded: Signal<HashSet<String>>,
    expanded: Signal<HashSet<String>>,
    pending: Signal<HashSet<String>>,
) -> Element {
    use_drop(move || {
        loaded.set(HashSet::new());
        expanded.set(HashSet::new());
        pending.set(HashSet::new());
    });
    rsx! {}
}

#[component]
pub fn TreePage() -> Element {
    let theme = use_theme();
    let mut current = use_signal(|| None::<String>);
    let mut expanded = use_signal(folders);
    let mut picked = use_signal(|| HashSet::from(["src/lib.rs".to_string()]));
    let mut checks = use_signal(|| HashSet::from(["src/components/list.rs".to_string()]));
    // demo-code: lazy start
    let mut lazy_expanded = use_signal(HashSet::<String>::new);
    let mut loaded = use_signal(HashSet::<String>::new);
    let mut pending = use_signal(HashSet::<String>::new);
    // Stands in for a fetch: whatever opened meanwhile arrives at once.
    let fetch = use_timeout(
        move || {
            let arrived = pending.take();
            loaded.write().extend(arrived);
        },
        800,
    );
    let mut on_lazy_expand = move |next: HashSet<String>| {
        let opened = next.iter().filter(|id| !loaded.read().contains(*id));
        let opened: Vec<String> = opened.cloned().collect();
        if !opened.is_empty() {
            pending.write().extend(opened);
            fetch.start();
        }
        lazy_expanded.set(next);
    };
    // demo-code: lazy end

    rsx! {
        DocPage {
            title: "Tree",
            source: "libero/src/components/navigation/tree",
            markdown: "/md/tree.md",
            properties: vec![
                props("Tree", vec![
                    prop("size", "Size")
                        .default(theme.tree.size.as_str())
                        .doc("Row gap and per-level indent together, from `List`'s scale. Set them apart through `sx`."),
                    prop("guides", "bool")
                        .default("false")
                        .doc("Draws a line down each open branch, under its chevron, and marks the `current` row's part of it. The lines use logical properties, so they move to the right under `dir=\"rtl\"`. Theme default `TreeDefaults::guides`."),
                    prop("aria_label", "String").default("required").doc("The tree's accessible name."),
                    prop("data", "Vec<TreeNode<T>>").default("required").doc("The nodes, over your own data type."),
                    prop("render_node", "Callback<TreeNodeRenderArgs<T>, Element>")
                        .default("default_tree_render")
                        .doc("Each visible row's content. A custom one can still call the default for some rows. `TreeItemContent` is the standard row and `TreeItem` a plain label the row names and Enter clicks. Both take `icon` and `trailing_icon` as `SvgData`, and only they draw the `checked` checkbox. A row that links somewhere is a `NavLink` with `args.tabindex`."),
                    prop("default_expanded", "HashSet<String>").doc("The ids expanded at first. Read once, and ignored when `expanded` is set."),
                    prop("expanded", "HashSet<String>").doc(
                        "The expanded ids, controlled. The tree follows it and asks for every change through `onexpandedchange`. Unset, the tree keeps its own.",
                    ),
                    prop("current", "String").doc(
                        "The id of the node the user is on, such as a nav's current page. Tab into the tree lands on it, or on its collapsed branch. Its row carries `aria-current` and, unless a `guides` line marks it, NavLink's tint and start bar. A row that renders a `NavLink` draws both, so pair one with `guides`.",
                    ),
                    prop("onexpandedchange", "EventHandler<HashSet<String>>")
                        .doc("Called with the whole new set of expanded ids. Store it when `expanded` is set; without `expanded` it only notifies."),
                    prop("selected", "HashSet<String>").doc(
                        "The selected ids, controlled: set, the rows are selectable and carry `aria-selected`. A click or Space picks a row. Store it with `onselectedchange`.",
                    ),
                    prop("multiple", "bool").default("false").doc(
                        "With `selected`, any number of rows: a click or Space toggles one, and Shift extends a range. Sets `aria-multiselectable` on the root.",
                    ),
                    prop("onselectedchange", "EventHandler<HashSet<String>>")
                        .doc("Called with the whole new set of selected ids. Store it when `selected` is set."),
                    prop("checked", "HashSet<String>").doc(
                        "The checked ids, controlled: set, every row draws a checkbox and carries `aria-checked`. Space or a click on the box toggles it, and so does a click on a leaf row while `selected` is unset. Store it with `oncheckedchange`.",
                    ),
                    prop("check_strictly", "bool").default("false").doc(
                        "With `checked`, each row checks alone. Unset, checking a branch checks its enabled descendants, and a branch reads `mixed` while only some are checked.",
                    ),
                    prop("oncheckedchange", "EventHandler<HashSet<String>>").doc(
                        "Called with the whole new set of checked ids. Store it when `checked` is set. Unset `check_strictly`, the set holds every fully checked node.",
                    ),
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
                    prop("children", "Vec<TreeNode<T>>").doc("Nested nodes. Without any, the node is a leaf, so a branch whose children load later holds a disabled placeholder row, such as \"Loading…\", until then. `onexpandedchange` says when the branch opens, and a focused placeholder hands focus to the first loaded row."),
                    prop("disabled", "bool").default("false").doc("The arrow keys still reach it, but it does not activate, expand, collapse, select or check. Its descendants are disabled too."),
                ]).without_base_props(),
                props("TreeNodeRenderArgs<T>", vec![
                    prop("id", "String").doc("The node's id."),
                    prop("data", "T").doc("The node's data."),
                    prop("expanded", "Option<bool>").doc("`None` for a leaf."),
                    prop("disabled", "bool").doc("Whether the node or an ancestor is disabled."),
                    prop("tabindex", "&'static str")
                        .doc("Put it on any link or button in the row, or it adds a tab stop the arrow keys never reach."),
                    prop("depth", "usize").doc("0 at the top level, for custom indentation."),
                    prop("selected", "bool").doc("In `Tree`'s `selected`. Always `false` without it."),
                    prop("checked", "Option<bool>").doc("With `checked`, the row's check; `None` without checkboxes. `TreeItemContent` and `TreeItem` draw the box themselves."),
                    prop("indeterminate", "bool").doc("Some of a branch's rows are checked, not all; `checked` is then `Some(false)`."),
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
                    prop("trailing", "Element").doc("After `trailing_icon`, such as a count. Never interactive: it is part of the activated label."),
                    prop("children", "Element").default("required").doc("The label."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters the tree, one tab stop.")
                .key(["Up", "Down"], "Moves between visible rows.")
                .key(["Left", "Right"], "Collapses and expands, or jumps to the parent and first child.")
                .key(["Home", "End"], "Jumps to the first or last row.")
                .key(["Enter"], "Opens or closes a branch, or clicks a leaf's link, button or `TreeItem`. In a single `selected` tree, a leaf is selected too. With both `selected` and `checked`, toggles the row's selection instead, branches included.")
                .key(["Space"], "Opens or closes a branch, or clicks a leaf's link, button or `TreeItem`. With `selected` or `checked`, toggles the row's selection or check instead; with both, it checks.")
                .key(["*"], "Opens every closed sibling of the row.")
                .key(["Shift+Space"], "With `selected` and `multiple`, adds the range from the last row picked to this one. Not with `checked`.")
                .key(["Shift+Up", "Shift+Down"], "With `selected` and `multiple`, moves focus and toggles the row it lands on.")
                .key(["Ctrl+A"], "With `selected` and `multiple`, selects every enabled node, collapsed ones too. Clears them when all are selected. Cmd+A on a Mac.")
                .key(["Ctrl+Shift+Home", "Ctrl+Shift+End"], "With `selected` and `multiple`, selects from this row to the first or last visible row, and moves focus there.")
                .handles([
                    "Typing jumps to the next row whose `tree_label` matches.",
                    "The arrow keys reach a disabled node, but typing skips it and nothing activates, expands or collapses it.",
                    "With `selected`, every row carries `aria-selected`, `true` or `false`. The root is `aria-multiselectable` with `multiple`.",
                    "With `checked`, every row carries `aria-checked`: `true`, `false`, or `mixed` for a branch with only some enabled rows checked (not with `check_strictly`).",
                    "A disabled node is never selected or checked, by keys, clicks or a branch's cascade.",
                    "`TreeItem` takes `args.tabindex` for you.",
                    "A `NavLink` in a disabled row is disabled with it and goes nowhere, unless it sets `disabled: false`.",
                ])
                .must([
                    "Name the tree with `aria_label`. It is required.",
                    "Pass `args.tabindex` to any link or button `render_node` draws, or it adds a tab stop the arrow keys never reach.",
                ])
                .example("A file browser, `Tree { aria_label: \"Project files\", .. }`: Tab enters on one row, Down walks the visible rows, Right opens a folder, and typing \"ma\" jumps to `main.rs`."),
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
                    ", as here. To load a branch's children when it opens, give it a disabled "
                    "placeholder row and swap in the children from "
                    Code { source: "onexpandedchange" }
                    ", as the lazy loading switch shows."
                }
            },
            // snippet: item #[derive(Clone, PartialEq)] struct FileEntry { name: &'static str }
            // snippet: item impl TreeLabel for FileEntry { fn tree_label(&self) -> String { self.name.to_string() } }
            // snippet: item fn file_tree() -> Vec<TreeNode<FileEntry>> { Vec::new() }
            // snippet: item fn file_icon(_: &FileEntry, _: Option<bool>) -> libero::components::SvgData { pictogram_icons_lucide::file::outlined }
            // snippet: item fn lazy_file_tree(_: &HashSet<String>) -> Vec<TreeNode<FileEntry>> { Vec::new() }
            // snippet: item use std::collections::HashSet;
            // snippet: item fn folders() -> HashSet<String> { HashSet::new() }
            // snippet: let mut current = use_signal(|| None::<String>);
            // snippet: let mut expanded = use_signal(folders);
            // snippet: let mut picked = use_signal(|| HashSet::from(["src/lib.rs".to_string()]));
            // snippet: let mut checks = use_signal(|| HashSet::from(["src/components/list.rs".to_string()]));
            Demo {
                component: "Tree",
                children_text: "",
                fixed: fixed(),
                controls: vec![
                    Control::switch("guides"),
                    Control::sizes("size")
                        .default(theme.tree.size.as_str()),
                    Control::toggle("selection", ["none", "single", "multiple"])
                        .default("multiple")
                        .code(selection_code),
                    Control::switch("checkboxes")
                        .default("true")
                        .code(checkbox_code),
                    Control::switch("check_strictly")
                        .hidden_when(|values: &DemoValues| values.str("checkboxes") != "true"),
                    // Branches fetch their children on opening; prints the data and expansion either way.
                    Control::switch("lazy_loading").code(loading_code),
                ],
                render: move |values: DemoValues| { let lazy_on = lazy(&values); rsx! {
                    Flex {
                        direction: "column",
                        gap: "sm",
                        // Pinned, so the selection readout does not reflow the
                        // preview under the pointer.
                        sx: sx().width("100%").max_width("320px"),
                        if lazy_on {
                            LazyReset { loaded, expanded: lazy_expanded, pending }
                        }
                        if !lazy_on {
                            // demo-code: buttons start
                            Flex { direction: "row", gap: "xs",
                                Button { size: "xs", onclick: move |_| expanded.set(folders()), "Expand all" }
                                Button { size: "xs", onclick: move |_| expanded.set(HashSet::new()), "Collapse all" }
                            }
                            // demo-code: buttons end
                        }
                        Tree {
                            // demo-code: wiring start
                            aria_label: "Project files",
                            current: current(),
                            // demo-code: wiring end
                            data: if lazy_on { lazy_file_tree(&loaded.read()) } else { file_tree() },
                            expanded: if lazy_on { lazy_expanded() } else { expanded() },
                            onexpandedchange: move |next| match lazy_on {
                                true => on_lazy_expand(next),
                                false => expanded.set(next),
                            },
                            guides: values.str("guides") == "true",
                            size: or_unset(values.str("size")),
                            selected: (values.str("selection") != "none").then(|| picked.read().clone()),
                            multiple: values.str("selection") == "multiple",
                            onselectedchange: move |next| picked.set(next),
                            checked: (values.str("checkboxes") == "true").then(|| checks.read().clone()),
                            oncheckedchange: move |next| checks.set(next),
                            check_strictly: values.str("check_strictly") == "true",
                            // demo-code: render start
                            render_node: move |args: TreeNodeRenderArgs<FileEntry>| {
                                let id = args.id.clone();
                                rsx! {
                                    TreeItem {
                                        onclick: move |_| current.set(Some(id.clone())),
                                        icon: file_icon(&args.data, args.expanded),
                                        "{args.data.name}"
                                    }
                                }
                            },
                            // demo-code: render end
                        }
                        // demo-code: status start
                        Text { role: "status",
                            if let Some(current) = current() {
                                "Current: "
                                Code { source: "{current}" }
                            }
                        }
                        // demo-code: status end
                        if lazy_on {
                            // demo-code: loading start
                            Text { role: "status", size: "sm",
                                if !pending.read().is_empty() {
                                    "Loading…"
                                }
                            }
                            // demo-code: loading end
                        }
                    }
                } },
                wrap: Wrap(wrap_selection),
            }
        }
    }
}
