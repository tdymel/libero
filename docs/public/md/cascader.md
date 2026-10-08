# Cascader

Crate: `libero`
Import: `use libero::components::Cascader;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/cascader/cascader.rs>
Index: [index.md](index.md) lists every other page
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

`node` draws a row's content from `CascaderNodeArgs`: its `level` (0 for a
root), whether it is `expanded` and whether it is `selected`. The highlight,
chevron and click stay the component's. In `"paths"` it draws each level of the
path. The icon and the check are decoration, hidden from screen readers:

```rust,ignore
Cascader {
    data: categories(),
    value: chosen(),
    onchange: move |next: Option<String>| chosen.set(next),
    node: move |n: CascaderNodeArgs<String>| {
        let icon = match n.value.as_str() {
            "food" => "🍽️",
            "drink" => "🥤",
            _ => "🧽",
        };
        rsx! {
            if n.level == 0 {
                span { "aria-hidden": "true", "{icon} " }
            }
            Text {
                component: "span",
                sx: sx().font_weight(if n.expanded { "600" } else { "inherit" }),
                "{n.label}"
            }
            if n.selected {
                span { "aria-hidden": "true", " ✓" }
            }
        }
    },
}
```

## Props

### `Cascader`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size of the frame and its rows. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `data` | `Vec<CascaderOption<T>>` | required | The tree, built with `CascaderOption::new(value, label)`, `.children(..)` and `.disabled(..)`. `T` is any `Options` type. Values must be unique across the whole tree. |
| `value` | `Option<T>` | - | The selected option's value. Pair it with `onchange`. The cascader finds the path to it in `data`, and a value no option holds selects nothing there: the trigger shows its `Options::label()` and the form still posts it, as `Select` does. |
| `onchange` | `EventHandler<Option<T>>` | - | Called with the value to select next, or `None` when the selection was cleared. |
| `any_level` | `bool` | `false` | Lets a branch be picked as well as expanded, as its own value. Off, only a leaf commits. |
| `allow_deselect` | `bool` | `false` | Picking the selected option again clears it. Off, a re-pick keeps the value. |
| `layout` | `CascaderLayout` | `columns` | `"columns"` draws one list per level, `"paths"` one row per full path. A search always renders `"paths"`. Without `any_level` it lists only leaf paths. On a screen narrower than the `sm` breakpoint, `"columns"` shows one level at a time under a back header. |
| `searchable` | `bool` | `false` | Puts a search box at the top of the list, which narrows it to the paths that match. |
| `filter` | `Callback<CascaderFilterArgs<T>, bool>` | - | Narrows the paths while searching. Defaults to a case-insensitive `contains` over the joined path. |
| `separator` | `String` | `" / "` | Between labels, in the trigger and in a `"paths"` row. |
| `format_value` | `Callback<Vec<String>, String>` | - | Replaces the joined labels in the trigger. Gets the labels from root to option, and returns a `String` so the trigger can still cut it off with an ellipsis. |
| `node` | `Callback<CascaderNodeArgs<T>, Element>` | `label` | Draws one row's content. The highlight, chevron and click stay the component's. |
| `column_width` | `String` | `220px` | Width and minimum width of one column. A trigger wider than the open columns shares the rest among them. `"max-content"` fits the longest row. On a narrow screen, the minimum width of the one level shown. |
| `name` | `FieldName<Option<T>>` | - | Posts the selected value's `Options::value()` in a hidden input of that name. A path such as `Listing::FIELDS.category()` also binds the selection to the surrounding `Form`'s value when there is no `onchange`. |
| `validate` | `Validators<Option<T>>` | - | Rules over the selected value, shown once the field loses focus or its form is submitted. |
| `placeholder` | `String` | - | Shown while nothing is selected. |
| `search_placeholder` | `String` | - | What the search box says while empty. |
| `clearable` | `bool` | `false` | Shows an x in place of the chevron while a value is selected. |
| `label` | `Caption` | - | The caption above the control, and the field's name. |
| `description` | `Caption` | - | Between the label and the control. What to pick. |
| `helper` | `Caption` | - | Under the control. What the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. |
| `dropdown_parts` | `Parts<DropdownPart>` | - | Styles the portaled dropdown and its inner parts. |

`CascaderNodeArgs<T>` carries `value`, `label`, `level`, `expanded` and
`selected`. `CascaderFilterArgs<T>` carries `query`, `label` and `path`, the
values from root to option.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the trigger.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `CascaderPart::Label` | `label` | The label above the control. |
| `CascaderPart::Required` | `required` | The required asterisk, in the label. |
| `CascaderPart::Description` | `description` | The caption between the label and the control. |
| `CascaderPart::Frame` | `frame` | The bordered box around the control. |
| `CascaderPart::Control` | `control` | The element the label names. |
| `CascaderPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `CascaderPart::Value` | `value` | The joined path or the placeholder, in the trigger. |
| `CascaderPart::Helper` | `helper` | The caption under the control. |
| `CascaderPart::Status` | `status` | The validation message. |

### Dropdown

The dropdown is portaled out of the field, so its parts take the
`dropdown_parts` prop. They match from the dropdown box at any depth.

| Part | `data-slot` | Description |
|---|---|---|
| `DropdownPart::Panel` | `dropdown` | The dropdown box itself. |
| `DropdownPart::Search` | `search` | The search box above the rows. |
| `DropdownPart::Listbox` | `listbox` | The scrolling list of rows. |
| `DropdownPart::Option` | `option` | A row. |
| `DropdownPart::OptionLabel` | `label` | A row's `span { "data-slot": "label" }`, which ends in an ellipsis. |
| `DropdownPart::Empty` | `nothing-found` | The text shown when the query matches nothing. |
| `DropdownPart::Column` | `column` | One level's column. |
| `DropdownPart::DrillBack` | `drill-back` | On a narrow screen, the header over a child level that goes back to its parent. |
| `DropdownPart::PickParent` | `pick-parent` | On a narrow screen with `any_level`, the row that picks the parent. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down`, `Up`, `Right`, `Enter` or `Space` | Closed: opens on the committed path, or with the cursor on the first enabled root (`Up`: the last). |
| `Home` or `End` | Closed: opens with the cursor on the first or last enabled root. Open: moves to the first or last enabled row of the column. |
| `Letter` | Unless searchable: moves to the next enabled row of the column starting with the typed text. A closed list opens on the roots. |
| `Down` or `Up` | Open: moves within the column, skipping disabled rows. |
| `Right` | Open, in `"columns"`: expands the row, the cursor onto its first enabled child. |
| `Left` | Open, in `"columns"`: up one level. At the root, nothing. |
| `Enter` | Open, on a leaf: picks it and closes. On the committed leaf it keeps the value, or clears it with `allow_deselect`. |
| `Enter` | Open, on a branch: expands it. Picks it too, with `any_level`. |
| `Space` | Open, unless searchable: as `Enter`. |
| `Tab` or `Alt+Up` | Open: picks the cursor's row if `Enter` would, and closes. `Tab` moves on. |
| `Escape` | Open: closes and keeps the value. |

### Libero handles

- In `"paths"`, and so while searching, `Left` and `Right` move the search
  box's caret.
- Below the `sm` breakpoint (48rem), `"columns"` shows only the cursor's
  level. A header names its parent, and its back button ("Back to Europe",
  `Localization::cascader.back`) goes up one level, as `Left` does. It never
  takes focus, so focus stays on the trigger. With `any_level`, a first row
  "Select Europe" picks the parent and closes. The keyboard cursor does not
  reach that row: `Left`, then `Enter` on the parent, picks it.
- Below the `sm` breakpoint the dropdown is a full-width sheet at the foot of
  the screen. It is not modal: no backdrop, no focus trap, and Escape or a
  press outside closes it. Opening it scrolls the page, as far as it can, so
  the trigger stays above the sheet.
- Android's Back button closes the dropdown as Escape does, rather than the
  app.
- The clear button is named by the field's `label`, "Clear Fruit", so two clear
  buttons on one form tell apart. Without a `label` it is "Clear" alone.

### You must

- Without a `label`, set `aria_label`. Otherwise screen readers announce an
  unnamed combobox.

### Example

A category picker, `Cascader { label: "Category" }`: Down opens it, Right
expands a branch, Left goes up a level, Enter on a leaf picks it and closes,
and Escape closes without a change.

## Theme defaults

`theme.cascader` is `CascaderDefaults { size, radius, column_width }`. The frame
comes from `FieldDefaults` and the list from `ComboboxDefaults`, so a cascader
lines up with a `TextField` and a `Select`.
