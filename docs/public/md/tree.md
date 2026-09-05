# Tree

Crate: `libero`
Import: `use libero::components::{Tree, TreeItem, TreeLabel, TreeNode, TreeNodeRenderArgs};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/tree>
Index: [index.md](index.md) - every other component's markdown page
Description: A data-driven, keyboard-navigable tree view over your own node type.

Data-driven, not composed via children - pass `Vec<TreeNode<T>>` where `T` is
entirely your own data shape. `T` only needs to implement `TreeLabel` (one
method, `fn tree_label(&self) -> String`), which is used for keyboard typeahead
and as the default row rendering. A `String` already implements it, so a plain
tree needs nothing further - and `default_tree_render(args)` gives one row that
rendering back.

Which nodes are expanded is `Tree`'s own business, not the caller's -
`default_expanded` only seeds the initial state. Selection and what happens on
click, though, are entirely `render_node`'s call: `Tree` doesn't assume clicking
a leaf means "select it" - a leaf might just as well be a real link. The closure
also gets the row's live `expanded` state, so content can react to it.

`TreeItem` is the row content a clickable leaf wants: the `button` Enter/Space
activation looks for, with its chrome stripped and the page's type inherited. It
takes the row's tab stop and `disabled` from the row itself, so neither can be
forgotten. A row that is a real link is a [`NavLink`](nav_link.md) instead - pass
it `args.tabindex` yourself.

`Tree` renders through [`List`](list.md) and has no size scale of its own: `size`
is the row gap and the per-level indent together. Off-scale, or the two apart,
goes through `sx` - rows are yours, so their type scale is too.

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
            sx: sx().width("320px"),
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

Selection is the caller's state, not `Tree`'s - the readout above is part of the
example, not something the component provides.

## Accessibility

`aria_label` is required - WAI-ARIA's tree pattern needs a name.

Arrow keys move between visible rows, `Left`/`Right` collapse/expand (or jump
to the parent/first child), `Home`/`End` jump to the first/last row, and typing
a letter jumps to the next row whose `tree_label` matches. A disabled node is
still reachable by the arrows, but nothing activates, expands or collapses it.

The tree is one roving tab stop, so any interactive element `render_node`
renders must take `args.tabindex`, or it becomes a second stop the arrow keys
never move to. `TreeItem` does that for you.

## Props

### `Tree<T>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Row gap and per-level indent together - `List`'s scale, since `Tree` renders through it. Off-scale, or the two apart, goes through `sx`. |
| `aria_label` | `String` | - | Required by WAI-ARIA's tree pattern. |
| `data` | `Vec<TreeNode<T>>` | required | The tree's data, entirely your own shape. |
| `render_node` | `Callback<TreeNodeRenderArgs<T>, Element>` | `default_tree_render` | Each visible row's content. The default can also be called selectively - say, default branches and `NavLink` leaves. |
| `default_expanded` | `HashSet<String>` | - | Seeds `Tree`'s internal state once. Not a controlled prop. |
| `onexpandedchange` | `EventHandler<HashSet<String>>` | - | Notification only - it doesn't drive rendering. |

Like every component, `Tree` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

### `TreeNode<T>`

| Field | Type | Default | Description |
|---|---|---|---|
| `id` | `String` | required | The node's identity, used for expansion state and keyboard navigation. |
| `data` | `T` | required | The caller's own data for this node. |
| `children` | `Vec<TreeNode<T>>` | - | Nested nodes - an empty vec makes this a leaf. |
| `disabled` | `bool` | `false` | Reachable by the arrow keys; nothing activates, expands or collapses it. |

### `TreeNodeRenderArgs<T>`

| Field | Type | Description |
|---|---|---|
| `id` | `String` | The node's id. |
| `data` | `T` | The node's data. |
| `expanded` | `bool` | `None` for a leaf - no children, no chevron, no `aria-expanded`. |
| `disabled` | `bool` | Whether the node is disabled. |
| `tabindex` | `&'static str` | Apply to any interactive element your content renders - the row is already the roving tab stop, without this a link/button adds a second one arrow keys never move. |
| `depth` | `usize` | 0 at top level. Lets `render_node` take indentation over entirely. |

### `TreeItem`

| Prop | Type | Default | Description |
|---|---|---|---|
| `onclick` | `EventHandler<MouseEvent>` | - | Fires on click. |
| `children` | `Element` | required | The row's content - an icon, the label. |

## Theme defaults

`TreeDefaults` on the theme. `Tree` renders through `List`, so this only picks
which of `List`'s levels it defaults to - the gap and indent values themselves
live in `ListDefaults`.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |

## CSS variables

`Tree` declares none of its own; sizing comes from `List`'s
`--lsx-list-gap-<size>` and `--lsx-list-indent-<size>`.

## Data attributes

| Token | Condition |
|---|---|
| `disabled` | On a row whose node is disabled. |

The root also carries whatever `data-state` tokens `List` sets for its `size`.
