# Cascader

Crate: `libero`
Import: `use libero::components::Cascader;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/cascader/cascader.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A field for choosing one branch of a tree level by level, whose value is the `Vec<String>` path from the root to the picked node.

A field for choosing one branch of a tree, level by level, with the five slots
every field shares. Its value is the path - the ids from the root down to the
picked node - and `onchange` hands back the nodes on that path beside it.

That value is what it is for. [Select](select.md) holds one `T`,
[MultiSelect](multi_select.md) a `Vec<T>` whose order means nothing, and
[Tree](tree.md) holds expansion state and *activates* a node rather than
selecting one. A cascader is the only one of them whose answer is "which
branch".

It takes `Tree`'s own `TreeNode<T>`, so the same data drives both.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Cascader, CascaderPick, Flex, Text, TreeNode};

fn categories() -> Vec<TreeNode<&'static str>> {
    vec![
        TreeNode::new("food", "Food").children(vec![
            TreeNode::new("fruit", "Fruit").children(vec![
                TreeNode::new("apple", "Apple"),
                TreeNode::new("pear", "Pear"),
            ]),
            TreeNode::new("veg", "Veg").children(vec![TreeNode::new("leek", "Leek")]),
        ]),
        TreeNode::new("drink", "Drink").children(vec![TreeNode::new("tea", "Tea")]),
    ]
}

#[component]
fn Demo() -> Element {
    // The value is the ids from the root to the picked node.
    let mut chosen = use_signal(|| vec!["drink".to_string(), "tea".to_string()]);

    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            Cascader {
                label: "Category",
                placeholder: "Pick a category",
                data: categories(),
                searchable: true,
                value: chosen(),
                onchange: move |pick: CascaderPick<&'static str>| chosen.set(pick.path),
            }
            // Prints `value: ["drink", "tea"]`.
            Text { size: "sm", "value: {chosen():?}" }
        }
    }
}
```

Strictly controlled: `value` is the path of ids, `onchange` hands back the path
the caller should hold next. An empty `path` is the cleared selection - what the
`clearable` x and `allow_deselect` both produce.

`CascaderPick<T>` carries both halves of one answer:

| Field | Type | What it is |
|---|---|---|
| `path` | `Vec<String>` | The node ids, root to leaf. What posts, and what `value` takes back |
| `nodes` | `Vec<T>` | The `data` of each node on that path, already resolved |

`nodes` exists so that "the label of the second level" is a field access rather
than a second walk of the tree.

Ids are unique across the **whole** tree, not just among siblings - the value is
a path of them, and a `value` that is not a path through `data` selects nothing
and warns.

## Two layouts

| `layout` | What the open list is |
|---|---|
| `"columns"` (default) | One `role="listbox"` per level, side by side. The deepest highlighted node is not expanded, so the columns never run ahead of the cursor |
| `"paths"` | One row per full path, labels joined by `separator` |

`searchable` renders `"paths"` whatever `layout` says: a search box sits at the
top of the list and narrows it to the paths that match, case-insensitively over
the joined path. `filter` replaces that rule and is handed the query, the joined
label, the ids and the resolved nodes.

Without `any_level` only a leaf can be committed, and `"paths"` lists only leaf
paths - so the list never offers a row `Enter` would refuse. With `any_level` a
branch commits as well as expanding, and every node gets a row.

A node under a disabled ancestor is disabled too; the arrows skip it and it
cannot be picked.

## Keyboard

| Key | State | Effect |
|---|---|---|
| `ArrowDown` / `ArrowUp` / `ArrowRight` / `Enter` / `Space` | closed | Opens. Down and Up seed the cursor on the first/last enabled root |
| `ArrowDown` / `ArrowUp` | open | Moves within the cursor's column, skipping disabled rows |
| `Home` / `End` | open | The first/last enabled row of that column |
| `ArrowRight` | open, `"columns"` | Expands the cursor's node, cursor onto its first enabled child |
| `ArrowLeft` | open, `"columns"` | Up one level. At the root, nothing |
| `Enter` | open, leaf | Commits the path and closes |
| `Enter` | open, branch | Expands. Commits too, with `any_level` |
| `Escape` | open | Closes and keeps the value |
| `Tab` | open | Closes and moves on |
| `Space` | open, not searchable | Swallowed, so the page does not scroll |

In `"paths"` - and so while searching - `ArrowLeft` and `ArrowRight` are left
alone, so they move the search box's caret.

## Accessibility

Focus never leaves the trigger. Each column is its own `role="listbox"` whose
rows are never focused, and the trigger names the row the arrows are on with
`aria-activedescendant` - the same pattern [Select](select.md) and
[Combobox](combobox.md) use, one cursor over several lists instead of one. A
column past the first is named by the row it hangs off, so nothing has to invent
a label for "level 2".

`aria-selected` follows the **cursor's own chain** rather than the committed
path. With several columns open those are routinely different rows, and marking
only the committed one would leave the `aria-activedescendant` target with no
`aria-selected` at all. The committed path keeps a mark of its own, in weight
rather than in ARIA.

While `searchable` and open, the search box owns `role="combobox"`,
`aria-controls` and `aria-activedescendant`; the trigger keeps only
`aria-haspopup` and `aria-expanded`, because two elements cannot both be the
combobox.

## Props

Everything `field_props!` gives every field - `label`, `description`, `helper`,
`status`, `size`, `radius`, `required`, `disabled`, `class`, `sx`, `states`,
`attributes` - plus:

| Prop | Type | Default | What it does |
|---|---|---|---|
| `data` | `Vec<TreeNode<T>>` | - | The tree. Ids unique across the whole tree |
| `value` | `Vec<String>` | `[]` | The selected path, root to leaf. Controlled |
| `onchange` | `EventHandler<CascaderPick<T>>` | - | The path to select next, and the nodes on it |
| `any_level` | `bool` | `false` | A branch commits as well as expanding |
| `allow_deselect` | `bool` | `true` | Picking the committed path again clears it |
| `layout` | `CascaderLayout` | `"columns"` | `"columns"` or `"paths"` |
| `searchable` | `bool` | `false` | A search box at the top of the list |
| `filter` | `Callback<CascaderFilterArgs<T>, bool>` | - | Defaults to case-insensitive `contains` over the joined path |
| `separator` | `String` | `" / "` | Between labels, in the trigger and in a `"paths"` row |
| `format_value` | `Callback<Vec<T>, String>` | - | Overrides the joined labels in the trigger |
| `node` | `Callback<CascaderNodeArgs<T>, Element>` | `tree_label()` | Draws one row's content |
| `column_width` | `String` | `"220px"` | One column's width. `"max-content"` fits the longest row |
| `clearable` | `bool` | `false` | An x in place of the chevron while a path is selected |
| `placeholder` | `String` | - | Shown while nothing is selected |
| `search_placeholder` | `String` | - | What the search box says while empty |
| `name` | `FieldName<Vec<String>>` | - | Posts one hidden input per level, and binds |
| `validate` | `Validators<Vec<String>>` | empty | Rules over the path |

`CascaderNodeArgs<T>` is `{ data, level, expanded, selected }`;
`CascaderFilterArgs<T>` is `{ query, label, path, nodes }`.

## Theme

`theme.cascader` is `CascaderDefaults { size, radius, column_width }`. The
frame's numbers come from `FieldDefaults` and the dropdown's - its padding, its
rows, its `max_dropdown_height` - from `ComboboxDefaults`, so a cascader lines up
with a `TextField` above it and with a `Select`'s list below it by construction.

## Not in scope

Multi-select with tri-state parents, and lazily loaded children. Mantine's
cascader ships without either, and both are their own component's semantics on
top of the most expensive control in the library.
