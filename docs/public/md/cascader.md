# Cascader

Crate: `libero`
Import: `use libero::components::Cascader;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/cascader/cascader.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A field for choosing one option of a tree level by level. Its value is the picked option's `value`, any `T: Options`; the cascader finds the path to it itself.

A field for choosing one option of a tree, level by level, with the five slots
every field shares. Its value is the picked option's `value`: any `T` a
[Select](select.md) could hold (`T: Options`), a `String` or a type of your own. The cascader finds the path to that value in `data`
itself, joins the labels on it in the trigger, and opens the columns on it.

What it adds over a `Select` is the walk: the options are a tree, reached one
level at a time. [Tree](tree.md) holds expansion state and *activates* a node
rather than selecting one, and the two share no data types - a cascader's tree
is built from its own `CascaderOption<T>`, Mantine's shape.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Cascader, CascaderOption, Flex, Text};

fn categories() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("food", "Food").children(vec![
            CascaderOption::new("fruit", "Fruit").children(vec![
                CascaderOption::new("apple", "Apple"),
                CascaderOption::new("pear", "Pear").disabled(true),
            ]),
            CascaderOption::new("veg", "Veg").children(vec![CascaderOption::new("leek", "Leek")]),
        ]),
        CascaderOption::new("drink", "Drink").children(vec![CascaderOption::new("tea", "Tea")]),
    ]
}

#[component]
fn Demo() -> Element {
    // The value is one option's value; the path to it is the cascader's to find.
    let mut chosen = use_signal(|| Some("tea".to_string()));

    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            Cascader {
                label: "Category",
                placeholder: "Pick a category",
                data: categories(),
                searchable: true,
                value: chosen(),
                onchange: move |next: Option<String>| chosen.set(next),
            }
            // The trigger shows `Drink / Tea`; this prints `value: Some("tea")`.
            Text { size: "sm", "value: {chosen():?}" }
        }
    }
}
```

Strictly controlled: `value` is the selected option's value, `onchange` hands
back the value the caller should hold next. `None` is the cleared selection -
what the `clearable` x and `allow_deselect` both produce.

`CascaderOption<T> { value: T, label: String, children, disabled }` is built
with `CascaderOption::new(value, label)`, `.children(vec![..])` and
`.disabled(true)`; `new` takes `impl Into<T>`, so a `CascaderOption<String>`
takes a `&str`. A disabled option disables everything under it.

The path to the value is found with `==`, the trigger shows the options'
own `label`s, and the hidden input posts `Options::value()` - what a `Select`
posts. Only a thin shell is generic over `T`; the engine walks the tree by
index path, so a second `T` does not compile a second engine.

Values are unique across the **whole** tree, not just among siblings: the
cascader finds its value by searching the tree, and a `value` no option holds
selects nothing and warns.

## Two layouts

| `layout` | What the open list is |
|---|---|
| `"columns"` (default) | One `role="listbox"` per level, side by side. The deepest highlighted node is not expanded, so the columns never run ahead of the cursor |
| `"paths"` | One row per full path, labels joined by `separator` |

`searchable` renders `"paths"` whatever `layout` says: a search box sits at the
top of the list and narrows it to the paths that match, case-insensitively over
the joined path. `filter` replaces that rule and is handed the query, the joined
label, and the path's values root to option.

Without `any_level` only a leaf can be committed, and `"paths"` lists only leaf
paths - so the list never offers a row `Enter` would refuse. With `any_level` a
branch commits its own value as well as expanding, and every option gets a row.

A node under a disabled ancestor is disabled too; the arrows skip it and it
cannot be picked.

## Accessibility

| Key | State | Effect |
|---|---|---|
| `ArrowDown` / `ArrowUp` / `ArrowRight` / `Enter` / `Space` | closed | Opens. Down and Up seed the cursor on the first/last enabled root |
| `ArrowDown` / `ArrowUp` | open | Moves within the cursor's column, skipping disabled rows |
| `Home` / `End` | open | The first/last enabled row of that column |
| `ArrowRight` | open, `"columns"` | Expands the cursor's node, cursor onto its first enabled child |
| `ArrowLeft` | open, `"columns"` | Up one level. At the root, nothing |
| `Enter` | open, leaf | Commits its value and closes |
| `Enter` | open, branch | Expands. Commits too, with `any_level` |
| `Escape` | open | Closes and keeps the value |
| `Tab` | open | Closes and moves on |
| `Space` | open, not searchable | Swallowed, so the page does not scroll |

In `"paths"` - and so while searching - `ArrowLeft` and `ArrowRight` are left
alone, so they move the search box's caret.

Without a `label` the trigger has no name of its own: set `aria_label`, or a
screen reader announces an unnamed combobox.

## Props

Every field's shared props - `label`, `description`, `helper`,
`status`, `size`, `radius`, `required`, `disabled`, `readonly`, `class`, `sx`, `states`,
`attributes` - plus:

| Prop | Type | Default | What it does |
|---|---|---|---|
| `data` | `Vec<CascaderOption<T>>` | - | The tree. Values unique across the whole tree |
| `value` | `Option<T>` | `None` | The selected option's value. Controlled |
| `onchange` | `EventHandler<Option<T>>` | - | The value to select next; `None` clears |
| `any_level` | `bool` | `false` | A branch commits its own value as well as expanding |
| `allow_deselect` | `bool` | `true` | Picking the selected option again clears it |
| `layout` | `CascaderLayout` | `"columns"` | `"columns"` or `"paths"` |
| `searchable` | `bool` | `false` | A search box at the top of the list |
| `filter` | `Callback<CascaderFilterArgs<T>, bool>` | - | Defaults to case-insensitive `contains` over the joined path |
| `separator` | `String` | `" / "` | Between labels, in the trigger and in a `"paths"` row |
| `format_value` | `Callback<Vec<String>, String>` | - | Overrides the joined labels in the trigger; takes the labels root to option |
| `node` | `Callback<CascaderNodeArgs<T>, Element>` | the label | Draws one row's content |
| `column_width` | `String` | `"220px"` | One column's width, and its minimum: when the trigger is wider than the open columns, they share the rest. `"max-content"` fits the longest row |
| `clearable` | `bool` | `false` | An x in place of the chevron while a value is selected |
| `placeholder` | `String` | - | Shown while nothing is selected |
| `search_placeholder` | `String` | - | What the search box says while empty |
| `name` | `FieldName<Option<T>>` | - | Posts one hidden input with `Options::value()`, and binds |
| `validate` | `Validators<Option<T>>` | empty | Rules over the selected value |

`CascaderNodeArgs<T>` is `{ value: T, label, level, expanded, selected }`;
`CascaderFilterArgs<T>` is `{ query, label, path: Vec<T> }`.

## Theme

`theme.cascader` is `CascaderDefaults { size, radius, column_width }`. The
frame's numbers come from `FieldDefaults` and the dropdown's - its padding, its
rows, its `max_dropdown_height` - from `ComboboxDefaults`, so a cascader lines up
with a `TextField` above it and with a `Select`'s list below it by construction.

## Not in scope

Multi-select with tri-state parents, and lazily loaded children. Mantine's
cascader ships without either, and both are their own component's semantics on
top of the most expensive control in the library.
