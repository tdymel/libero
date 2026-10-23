# Tree

Crate: `libero`
Import: `use libero::components::{Tree, TreeItem, TreeItemContent, TreeLabel, TreeNode, TreeNodeRenderArgs};`
Feature: `tree` (off by default, turns on `list`, `icon`, `use-popover`, `checkbox`; [features.md](features.md))
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
            // Always mounted, so a screen reader hears the text it gains.
            Text { role: "status",
                if let Some(selected) = selected() { "Selected: " Code { source: "{selected}" } }
            }
        }
    }
}
```

The expansion, the buttons and the selection readout are the example's own
state, not part of `Tree`.

## Rows

`render_node` decides what a row does, and gets the row's live `expanded`
state. `TreeItemContent` is the standard row: the chevron, an `icon`, the label
and a `trailing_icon`, icons as `SvgData` (see [`Pictogram`](pictogram.md)). The
default row is one. `TreeItem` lays a row out the same way as a plain label the
row names and Enter clicks (a button outside a `Tree`). A row that links
somewhere is a [`NavLink`](nav_link.md) with `args.tabindex`. The checkbox of
`checked` is drawn only by `TreeItemContent` and `TreeItem`, so a custom row
without them has none.
`default_tree_render(args)` draws the default row, so a custom `render_node`
can fall back to it.

## Guides

`guides` draws a line down each open branch, under its chevron, and marks the
`current` row's part of it, as this site's sidebar does. The lines use logical
properties, so they move to the right under `dir="rtl"`.

`Tree` renders through [`List`](list.md). `size` sets the row gap and the
per-level indent together. Set them apart through `sx`.

## Selection and checkboxes

`selected` makes the rows selectable, each with `aria-selected`. A click picks
a row: it toggles one, and replaces the rest unless `multiple` is set, where
Shift+click adds the visible rows from the last one picked. Like `expanded`,
the set is yours: the tree asks through `onselectedchange`. `checked` puts a
checkbox on each row, with `aria-checked`. Space or a click on the box toggles
it, and so does a click on a leaf row while `selected` is unset. Unless
`check_strictly` is set, checking a branch checks its enabled descendants, and
the branch reads `mixed` while only some of them are checked. Disabled nodes
are never selected or checked. The demo above uses both.

```rust
use std::collections::HashSet;

use dioxus::prelude::*;
use libero::components::{Tree, TreeNode};

#[component]
fn Demo() -> Element {
    let mut picked = use_signal(|| HashSet::from(["src/lib.rs".to_string()]));
    let mut checked = use_signal(HashSet::<String>::new);

    rsx! {
        Tree {
            aria_label: "Project files",
            data: vec![
                TreeNode::new("src", "src").children(vec![TreeNode::new("src/lib.rs", "lib.rs")]),
                TreeNode::new("README.md", "README.md"),
            ],
            default_expanded: HashSet::from(["src".to_string()]),
            selected: picked(),
            multiple: true,
            onselectedchange: move |next| picked.set(next),
            checked: checked(),
            oncheckedchange: move |next| checked.set(next),
        }
    }
}
```

## Lazy loading

A node without children is a leaf, which cannot open. To load a branch's
children when it opens, give it one disabled placeholder row, such as
"Loading…", and swap in the children once they arrive. `onexpandedchange` says
when the branch opens. A focused placeholder hands focus to the first loaded
row.

```rust
use std::collections::HashSet;

use dioxus::prelude::*;
use libero::components::{Tree, TreeNode};

#[component]
fn LazyFiles() -> Element {
    let mut expanded = use_signal(HashSet::<String>::new);
    let mut files = use_signal(|| None::<Vec<String>>);
    let children = match files() {
        Some(files) => files.into_iter().map(|file| TreeNode::new(format!("src/{file}"), file)).collect(),
        None => vec![TreeNode::new("src/loading", "Loading…".to_string()).disabled(true)],
    };

    rsx! {
        Tree {
            aria_label: "Files",
            data: vec![TreeNode::new("src", "src".to_string()).children(children)],
            expanded: expanded(),
            onexpandedchange: move |next: HashSet<String>| {
                if next.contains("src") && files.peek().is_none() {
                    // Your fetch goes here.
                    spawn(async move { files.set(Some(vec!["lib.rs".to_string(), "main.rs".to_string()])) });
                }
                expanded.set(next);
            },
        }
    }
}
```

## Props

### `Tree<T>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Row gap and per-level indent together, from `List`'s scale. Set them apart through `sx`. |
| `guides` | `bool` | `false` | Draws a line down each open branch, under its chevron, and marks the `current` row's part of it. Theme default `TreeDefaults::guides`. |
| `aria_label` | `String` | required | The tree's accessible name. |
| `data` | `Vec<TreeNode<T>>` | required | The nodes, over your own data type. |
| `render_node` | `Callback<TreeNodeRenderArgs<T>, Element>` | `default_tree_render` | Each visible row's content. A custom one can still call the default for some rows. `TreeItemContent` is the standard row and `TreeItem` a plain label. A row that links somewhere is a `NavLink` with `args.tabindex`. |
| `default_expanded` | `HashSet<String>` | - | The ids expanded at first. Read once, and ignored when `expanded` is set. |
| `expanded` | `HashSet<String>` | - | The expanded ids, controlled. The tree follows it and asks for every change through `onexpandedchange`. Unset, the tree keeps its own. |
| `current` | `String` | - | The id of the node the user is on, such as a nav's current page. Tab into the tree lands on it, or on its collapsed branch. Its row carries `aria-current` and, unless a `guides` line marks it, NavLink's tint and start bar. A row that renders a `NavLink` draws both, so pair one with `guides`. |
| `onexpandedchange` | `EventHandler<HashSet<String>>` | - | Called with the whole new set of expanded ids. Store it when `expanded` is set; without `expanded` it only notifies. |
| `selected` | `HashSet<String>` | - | The selected ids, controlled: set, the rows are selectable and carry `aria-selected`. A click or Space picks a row. Store it with `onselectedchange`. |
| `multiple` | `bool` | `false` | With `selected`, any number of rows: a click or Space toggles one, and Shift extends a range. Sets `aria-multiselectable` on the root. |
| `onselectedchange` | `EventHandler<HashSet<String>>` | - | Called with the whole new set of selected ids. Store it when `selected` is set. |
| `checked` | `HashSet<String>` | - | The checked ids, controlled: set, every row draws a checkbox and carries `aria-checked`. Space or a click on the box toggles it. Store it with `oncheckedchange`. |
| `check_strictly` | `bool` | `false` | With `checked`, each row checks alone. Unset, checking a branch checks its enabled descendants, and a branch reads `mixed` while only some are checked. |
| `oncheckedchange` | `EventHandler<HashSet<String>>` | - | Called with the whole new set of checked ids. Store it when `checked` is set. Unset `check_strictly`, the set holds every fully checked node. |
| `parts` | `Parts<TreePart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Tree` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

### `TreeNode<T>`

`TreeNode::new(id, data)`, plus `.children(..)` and `.disabled(bool)`.

| Field | Type | Default | Description |
|---|---|---|---|
| `id` | `String` | required | The node's unique id. |
| `data` | `T` | required | Your data for this node. |
| `children` | `Vec<TreeNode<T>>` | - | Nested nodes. Without any, the node is a leaf, so a branch whose children load later holds a disabled placeholder row until then. |
| `disabled` | `bool` | `false` | The arrow keys still reach it, but it does not activate, expand, collapse, select or check. Its descendants are disabled too. |

### `TreeNodeRenderArgs<T>`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | The node's id. |
| `data` | `T` | The node's data. |
| `expanded` | `Option<bool>` | `None` for a leaf. |
| `disabled` | `bool` | Whether the node or an ancestor is disabled. |
| `tabindex` | `&'static str` | Put it on any link or button in the row, or it adds a tab stop the arrow keys never reach. |
| `depth` | `usize` | 0 at the top level, for custom indentation. |
| `selected` | `bool` | In `Tree`'s `selected`. Always `false` without it. |
| `checked` | `Option<bool>` | With `checked`, the row's check; `None` without checkboxes. `TreeItemContent` and `TreeItem` draw the box themselves. |
| `indeterminate` | `bool` | Some of a branch's rows are checked, not all; `checked` is then `Some(false)`. |

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

An activatable label laid out like `TreeItemContent`, chevron included: inside a `Tree` the treeitem is the widget (no nested button), outside one it is a button.

| Prop | Type | Default | Description |
|---|---|---|---|
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler. |
| `icon` | `SvgData` | - | Drawn after the chevron, before the label. |
| `trailing_icon` | `SvgData` | - | Drawn at the row's end. |
| `trailing` | `Element` | - | After `trailing_icon`, such as a count. Never interactive: it is part of the activated label. |
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
| `Enter` | Opens or closes a branch, or clicks a leaf's link, button or `TreeItem`. In a single `selected` tree, a leaf is selected too. |
| `Space` | Opens or closes a branch, or clicks a leaf's link, button or `TreeItem`. With `selected` or `checked`, toggles the row's selection or check instead. |
| `*` | Opens every closed sibling of the row. |
| `Shift+Space` | With `selected` and `multiple`, adds the range from the last row picked to this one. Not with `checked`. |
| `Shift+Up` or `Shift+Down` | With `selected` and `multiple`, moves focus and toggles the row it lands on. |
| `Ctrl+A` (`Cmd+A` on a Mac) | With `selected` and `multiple`, selects every enabled node, collapsed ones too. Clears them when all are selected. |
| `Ctrl+Shift+Home` or `Ctrl+Shift+End` | With `selected` and `multiple`, selects from this row to the first or last visible row, and moves focus there. |

### Libero handles

- Typing jumps to the next row whose `tree_label` matches.
- The arrow keys reach a disabled node, but typing skips it and nothing
  activates, expands or collapses it.
- With `selected`, every row carries `aria-selected`, `true` or `false`. The
  root is `aria-multiselectable` with `multiple`.
- With `checked`, every row carries `aria-checked`: `true`, `false`, or `mixed`
  for a branch with only some enabled rows checked (not with `check_strictly`).
- A disabled node is never selected or checked, by keys, clicks or a branch's
  cascade.
- `TreeItem` takes `args.tabindex` for you.

### You must

- Name the tree with `aria_label`. It is required.
- Pass `args.tabindex` to any link or button `render_node` draws, or it adds a
  tab stop the arrow keys never reach.

### Example

A file browser, `Tree { aria_label: "Project files", .. }`: Tab enters on one
row, Down walks the visible rows, Right opens a folder, and typing "ma" jumps
to `main.rs`.

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
| `chevron_duration` | `u32` | Milliseconds for the chevron's turn, `120`. None under reduced motion. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-tree-guide-color` | The guide's colour. |
| `--lsx-tree-guide-active-color` | The `current` row's marker colour. |
| `--lsx-tree-guide-width` | The guide's width. |
| `--lsx-tree-guide-active-width` | The marker's width. |
| `--lsx-tree-chevron-duration` | The chevron's turn. |

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
| `selected` | On the content of a row that is selected. |

The root also carries whatever `data-state` tokens `List` sets for its `size`.
