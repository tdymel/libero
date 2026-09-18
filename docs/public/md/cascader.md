# Cascader

Crate: `libero`
Import: `use libero::components::Cascader;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/cascader/cascader.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A field that picks one option from a tree, one level at a time, and shows the path in the trigger.

Picks one option from a tree, one level at a time. The value is that option's
`value`, any `T` a [Select](select.md) could hold. The cascader finds the path
to it, shows the path in the trigger and opens the columns on it.

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

`value` is the selected option's value, and `onchange` hands back the next
one. `None` is the cleared selection, from the `clearable` x or from
`allow_deselect`.

`CascaderOption::new(value, label)` takes `impl Into<T>`, so a
`CascaderOption<String>` takes a `&str`. Add `.children(vec![..])` and
`.disabled(true)`. A disabled option disables everything under it, and the
arrows skip it.

Values must be unique across the whole tree, not just among siblings. The
cascader finds its value by searching the tree with `==`, and a value no option
holds selects nothing and warns. The hidden input posts `Options::value()`.

## Layouts

| `layout` | What the open list is |
|---|---|
| `"columns"` (default) | One `role="listbox"` per level, side by side. The columns open down to the highlighted row, never past it |
| `"paths"` | One row per full path, labels joined by `separator` |

`searchable` always renders `"paths"`. The search box narrows the list to the
paths that match, case-insensitively over the joined labels. `filter` replaces
that rule and gets the query, the joined label and the path's values.

Without `any_level` only a leaf can be picked, so `"paths"` lists only leaf
paths. With `any_level` a branch picks its own value as well as expanding, and
every option gets a row.

## Accessibility

| Key | State | Effect |
|---|---|---|
| `ArrowDown` / `ArrowUp` / `ArrowRight` / `Enter` / `Space` | closed | Opens on the committed path, or with the cursor on the first (Up: last) enabled root |
| `Home` / `End` | closed | Opens with the cursor on the first/last enabled root |
| A letter | not searchable | The next enabled row of the cursor's column starting with the typed text. A closed list opens on the roots |
| `ArrowDown` / `ArrowUp` | open | Moves within the cursor's column, skipping disabled rows |
| `Home` / `End` | open | The first/last enabled row of that column |
| `ArrowRight` | open, `"columns"` | Expands the cursor's node, cursor onto its first enabled child |
| `ArrowLeft` | open, `"columns"` | Up one level. At the root, nothing |
| `Enter` | open, leaf | Commits its value and closes. On the committed leaf it keeps the value (clears it with `allow_deselect`) |
| `Enter` | open, branch | Expands. Commits too, with `any_level` |
| `Space` | open, not searchable | As `Enter` |
| `Tab` / `Alt+ArrowUp` | open | Commits the cursor's row if `Enter` would, closes, and (Tab) moves on |
| `Escape` | open | Closes and keeps the value |

In `"paths"`, and so while searching, `ArrowLeft` and `ArrowRight` move the
search box's caret.

Without a `label`, set `aria_label`. Otherwise screen readers announce an
unnamed combobox.

## Props

### `Cascader`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size of the frame and its rows. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `data` | `Vec<CascaderOption<T>>` | - | The tree, built with `CascaderOption::new(value, label)`, `.children(..)` and `.disabled(..)`. `T` is any `Options` type. Values must be unique across the whole tree. |
| `value` | `Option<T>` | - | The selected option's value. Pair it with `onchange`. The cascader finds the path to it in `data`, and a value no option holds selects nothing. |
| `onchange` | `EventHandler<Option<T>>` | - | Called with the value to select next, or `None` when the selection was cleared. |
| `any_level` | `bool` | `false` | Lets a branch be picked as well as expanded, as its own value. Off, only a leaf commits. |
| `allow_deselect` | `bool` | `false` | Picking the selected option again clears it. Off, a re-pick keeps the value. |
| `layout` | `CascaderLayout` | `columns` | `"columns"` draws one list per level, `"paths"` one row per full path. A search always renders `"paths"`. |
| `searchable` | `bool` | `false` | Puts a search box at the top of the list, which narrows it to the paths that match. |
| `filter` | `Callback<CascaderFilterArgs<T>, bool>` | - | Narrows the paths while searching. Defaults to a case-insensitive `contains` over the joined path. |
| `separator` | `String` | `" / "` | Between labels, in the trigger and in a `"paths"` row. |
| `format_value` | `Callback<Vec<String>, String>` | - | Replaces the joined labels in the trigger. Gets the labels from root to option, and returns a `String` so the trigger can still cut it off with an ellipsis. |
| `node` | `Callback<CascaderNodeArgs<T>, Element>` | `label` | Draws one row's content. The highlight, chevron and click stay the component's. |
| `column_width` | `String` | `220px` | Width and minimum width of one column. A trigger wider than the open columns shares the rest among them. `"max-content"` fits the longest row. |
| `name` | `FieldName<Option<T>>` | - | Posts the selected value's `Options::value()` in a hidden input of that name. A path such as `Listing::FIELDS.category()` also binds the selection to the surrounding `Form`'s value when there is no `onchange`. |
| `validate` | `Validators<Option<T>>` | - | Rules over the selected value, shown once the field loses focus or its form is submitted. |
| `placeholder` | `String` | - | Shown while nothing is selected. |
| `search_placeholder` | `String` | - | What the search box says while empty. |
| `clearable` | `bool` | `false` | Shows an x in place of the chevron while a value is selected. |
| `label` | `Caption` | - | The caption above the control, and the field's name. |
| `description` | `Caption` | - | Between the label and the control. What to pick. |
| `helper` | `Caption` | - | Under the control. What the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. |

`CascaderNodeArgs<T>` carries `value`, `label`, `level`, `expanded` and
`selected`. `CascaderFilterArgs<T>` carries `query`, `label` and `path`, the
values from root to option.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the trigger.

## Theme defaults

`theme.cascader` is `CascaderDefaults { size, radius, column_width }`. The frame
comes from `FieldDefaults` and the list from `ComboboxDefaults`, so a cascader
lines up with a `TextField` and a `Select`.
