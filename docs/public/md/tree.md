# Tree

Crate: `libero`
Import: `use libero::components::{Tree, TreeItem, TreeLabel, TreeNode, TreeNodeRenderArgs};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/tree>
Index: [index.md](index.md) lists every other page
Description: A data-driven, keyboard-navigable tree view over your own node type.

A tree view over `Vec<TreeNode<T>>`, where `T` is your own data type. `T`
implements `TreeLabel`, whose text feeds typeahead and the default row.
`String` already does. The tree keeps its own expanded state, and
`default_expanded` seeds it.

`Tree` has no selection. `render_node` decides what a row does, and gets the
row's live `expanded` state. `TreeItem` makes a row a button that picks up the
tab stop and `disabled`. A row that links somewhere is a
[`NavLink`](nav_link.md) with `args.tabindex`. `default_tree_render(args)` draws
the default row, so a custom `render_node` can fall back to it.

`Tree` renders through [`List`](list.md). `size` sets the row gap and the
per-level indent together. Set them apart through `sx`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Code, Flex, Text, Tree, TreeItem, TreeLabel, TreeNode, TreeNodeRenderArgs};
use libero::sx::sx;

#[derive(Clone, PartialEq)]
struct FileEntry {
    name: &'static str,
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

#[component]
fn Demo() -> Element {
    let mut selected = use_signal(|| None::<String>);

    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            sx: sx().width("100%").max_width("320px"),
            Tree {
                aria_label: "Project files",
                data: file_tree(),
                default_expanded: ["src", "src/components"].map(str::to_string).into(),
                render_node: move |args: TreeNodeRenderArgs<FileEntry>| {
                    let id = args.id.clone();
                    rsx! {
                        TreeItem {
                            onclick: move |_| selected.set(Some(id.clone())),
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

The selection readout is the example's own state, not part of `Tree`.

## Accessibility

`aria_label` names the tree and is required.

The tree is one tab stop. Up and Down move between visible rows. Left and Right
collapse and expand, or jump to the parent and first child. Home and End jump
to the first and last row, and typing jumps to the next row whose `tree_label`
matches. A disabled node is still reachable, but nothing activates, expands or
collapses it.

Any link or button `render_node` draws must take `args.tabindex`, or it adds a
tab stop the arrow keys never reach. `TreeItem` does this for you.

## Props

### `Tree<T>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Row gap and per-level indent together, from `List`'s scale. Set them apart through `sx`. |
| `aria_label` | `String` | required | The tree's accessible name. |
| `data` | `Vec<TreeNode<T>>` | required | The nodes, over your own data type. |
| `render_node` | `Callback<TreeNodeRenderArgs<T>, Element>` | `default_tree_render` | Each visible row's content. A custom one can still call the default for some rows. |
| `default_expanded` | `HashSet<String>` | - | The ids expanded at first. Read once. |
| `current` | `String` | - | The id of the node the user is on, such as a nav's current page. Tab into the tree lands on it, or on its collapsed branch. Its row carries `aria-current`. |
| `onexpandedchange` | `EventHandler<HashSet<String>>` | - | Called with the expanded ids when they change. It does not control them. |

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

### `TreeItem`

| Prop | Type | Default | Description |
|---|---|---|---|
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler. |
| `children` | `Element` | required | The row's content, such as an icon and the label. |

## Theme defaults

`TreeDefaults` on the theme. It picks the default `List` level. The gap and
indent values live in `ListDefaults`.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size`, `md`. |

## CSS variables

None of its own. Sizing comes from `List`'s `--lsx-list-gap-<size>` and
`--lsx-list-indent-<size>`.

## Data attributes

| Token | Condition |
|---|---|
| `disabled` | On a row whose node is disabled. |

The root also carries whatever `data-state` tokens `List` sets for its `size`.
