# Tree

Crate: `libero`
Import: `use libero::components::{Tree, TreeItem, TreeItemContent, TreeLabel, TreeNode, TreeNodeRenderArgs};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/tree>
Index: [index.md](index.md) lists every other page
Description: A data-driven, keyboard-navigable tree view over your own node type.

A tree view over `Vec<TreeNode<T>>`, where `T` is your own data type. `T`
implements `TreeLabel`, whose text feeds typeahead and the default row.
`String` already does. The tree keeps its own expanded state, seeded by
`default_expanded`, or follows a controlled `expanded` paired with
`onexpandedchange`. Controlled, a key or click only asks: the branch opens
once you store the new set, and you can open one from outside, as this site's
sidebar opens the current page's section.

`Tree` has no selection. `render_node` decides what a row does, and gets the
row's live `expanded` state. `TreeItemContent` is the standard row: the
chevron, an `icon`, the label and a `trailing_icon`, icons as `SvgData` (see
[`Pictogram`](pictogram.md)). The default row is one. `TreeItem` lays a row out
the same way as a button that picks up the tab stop and `disabled`. A row that
links somewhere is a [`NavLink`](nav_link.md) with `args.tabindex`.
`default_tree_render(args)` draws the default row, so a custom `render_node`
can fall back to it.

`guides` draws a line down each open branch, under its chevron, and marks the
`current` row's part of it, as this site's sidebar does. The lines use logical
properties, so they move to the right under `dir="rtl"`.

`Tree` renders through [`List`](list.md). `size` sets the row gap and the
per-level indent together. Set them apart through `sx`.

## Usage

```rust
use std::collections::HashSet;

use dioxus::prelude::*;
use libero::components::{Button, Code, Flex, SvgData, Text, Tree, TreeItem, TreeLabel, TreeNode, TreeNodeRenderArgs};
use libero::sx::sx;
use pictogram_icons_lucide as lucide;

#[derive(Clone, PartialEq)]
struct FileEntry {
    name: &'static str,
}

fn file_icon(entry: &FileEntry, expanded: Option<bool>) -> SvgData {
    match expanded {
        Some(true) => lucide::folder_open::outlined,
        Some(false) => lucide::folder::outlined,
        None => lucide::file::outlined,
    }
}

impl TreeLabel for FileEntry {
    fn tree_label(&self) -> String {
        self.name.to_string()
    }
}

fn file_tree() -> Vec<TreeNode<FileEntry>> {
    vec![
        TreeNode::new("src", FileEntry { name: "src" }).children(vec![
            TreeNode::new("src/components", FileEntry { name: "components" }).children(vec![
                TreeNode::new("src/components/list.rs", FileEntry { name: "list.rs" }),
                TreeNode::new("src/components/tree.rs", FileEntry { name: "tree.rs" }),
            ]),
            TreeNode::new("src/hooks", FileEntry { name: "hooks" }),
            TreeNode::new("src/lib.rs", FileEntry { name: "lib.rs" }),
        ]),
        TreeNode::new("Cargo.toml", FileEntry { name: "Cargo.toml" }),
        TreeNode::new("README.md", FileEntry { name: "README.md" }),
    ]
}

/// Every branch of `file_tree()`.
fn folders() -> HashSet<String> {
    HashSet::from(["src", "src/components"].map(str::to_string))
}

#[component]
fn Demo() -> Element {
    let mut selected = use_signal(|| None::<String>);
    let mut expanded = use_signal(folders);

    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
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
                guides: true,
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
    }
}
```

The expansion, the buttons and the selection readout are the example's own
state, not part of `Tree`.

## Props

### `Tree<T>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Row gap and per-level indent together, from `List`'s scale. Set them apart through `sx`. |
| `guides` | `bool` | `false` | Draws a line down each open branch, under its chevron, and marks the `current` row's part of it. Theme default `TreeDefaults::guides`. |
| `aria_label` | `String` | required | The tree's accessible name. |
| `data` | `Vec<TreeNode<T>>` | required | The nodes, over your own data type. |
| `render_node` | `Callback<TreeNodeRenderArgs<T>, Element>` | `default_tree_render` | Each visible row's content. A custom one can still call the default for some rows. |
| `default_expanded` | `HashSet<String>` | - | The ids expanded at first. Read once, and ignored when `expanded` is set. |
| `expanded` | `HashSet<String>` | - | The expanded ids, controlled. The tree follows it and asks for every change through `onexpandedchange`. Unset, the tree keeps its own. |
| `current` | `String` | - | The id of the node the user is on, such as a nav's current page. Tab into the tree lands on it, or on its collapsed branch. Its row carries `aria-current`. |
| `onexpandedchange` | `EventHandler<HashSet<String>>` | - | Called with the whole new set of expanded ids. Store it when `expanded` is set; without `expanded` it only notifies. |
| `parts` | `Parts<TreePart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Tree` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

### `TreeNode<T>`

`TreeNode::new(id, data)`, plus `.children(..)` and `.disabled(bool)`.

| Field | Type | Default | Description |
|---|---|---|---|
| `id` | `String` | required | The node's unique id. |
| `data` | `T` | required | Your data for this node. |
| `children` | `Vec<TreeNode<T>>` | - | Nested nodes. Without any, the node is a leaf. |
| `disabled` | `bool` | `false` | The arrow keys still reach it, but it does not activate, expand or collapse. Its descendants are disabled too. |

### `TreeNodeRenderArgs<T>`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | The node's id. |
| `data` | `T` | The node's data. |
| `expanded` | `Option<bool>` | `None` for a leaf. |
| `disabled` | `bool` | Whether the node or an ancestor is disabled. |
| `tabindex` | `&'static str` | Put it on any link or button in the row, or it adds a tab stop the arrow keys never reach. |
| `depth` | `usize` | 0 at the top level, for custom indentation. |

### `TreeItemContent`

The chevron (a spacer on a leaf), `icon`, the label, `trailing_icon`, then
`trailing`, in one flex row that fills the tree row. Icons are `muted.6`, the
chevron's colour.

| Prop | Type | Default | Description |
|---|---|---|---|
| `icon` | `SvgData` | - | Drawn after the chevron, before the label. |
| `trailing_icon` | `SvgData` | - | Drawn at the row's end. |
| `trailing` | `Element` | - | After `trailing_icon`, such as a count. Never interactive inside a `TreeItem`. |
| `children` | `Element` | required | The label. |

### `TreeItem`

A button laid out like `TreeItemContent`, chevron included.

| Prop | Type | Default | Description |
|---|---|---|---|
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler. |
| `icon` | `SvgData` | - | Drawn after the chevron, before the label. |
| `trailing_icon` | `SvgData` | - | Drawn at the row's end. |
| `trailing` | `Element` | - | After `trailing_icon`, such as a count. Never interactive: it sits in the button. |
| `children` | `Element` | required | The label. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `TreePart::Row` | `row` | A `treeitem`: the row and its open subtree. A `Tree` nested in a row's content matches too. |
| `TreePart::Content` | `content` | The clickable line around `render_node`'s content. |
| `TreePart::Group` | `group` | An open branch's list of children. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Enters the tree, one tab stop. |
| `Up` or `Down` | Moves between visible rows. |
| `Left` or `Right` | Collapses and expands, or jumps to the parent and first child. |
| `Home` or `End` | Jumps to the first or last row. |

### Libero handles

- Typing jumps to the next row whose `tree_label` matches.
- A disabled node is still reachable, but nothing activates, expands or
  collapses it.
- `TreeItem` takes `args.tabindex` for you.

### You must

- Name the tree with `aria_label`. It is required.
- Pass `args.tabindex` to any link or button `render_node` draws, or it adds a
  tab stop the arrow keys never reach.

## Theme defaults

`TreeDefaults` on the theme. It picks the default `List` level. The gap and
indent values live in `ListDefaults`.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size`, `md`. |
| `guides` | `bool` | Default `guides`, `false`. |
| `guide_color` | `ColorValue` | The guide's colour, `muted.3`. |
| `guide_active_color` | `ColorValue` | The `current` row's marker, `primary.6`. |
| `guide_width` | `u8` | The guide's width in px, `1`. |
| `guide_active_width` | `u8` | The marker's width in px, `2`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-tree-guide-color` | The guide's colour. |
| `--lsx-tree-guide-active-color` | The `current` row's marker colour. |
| `--lsx-tree-guide-width` | The guide's width. |
| `--lsx-tree-guide-active-width` | The marker's width. |

Sizing comes from `List`'s `--lsx-list-gap-<size>` and
`--lsx-list-indent-<size>`. With `guides`, a level's indent is the guide's
offset plus what is left of the list indent, so it is never smaller than the
offset.

## Data attributes

| Token | Condition |
|---|---|
| `disabled` | On a row whose node is disabled. |
| `guides` | On every row of a tree with `guides`. |
| `guide-current` | On the content of a nested `current` row in a tree with `guides`. |

The root also carries whatever `data-state` tokens `List` sets for its `size`.
