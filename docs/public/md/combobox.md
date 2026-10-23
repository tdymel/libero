# Combobox

Crate: `libero`
Import: `use libero::components::{Combobox, ComboboxOption, ComboboxOptionArgs, use_combobox};`
Feature: `combobox` (on by default, turns on `loader`, `paper`, `scroll-area`, `use-id`, `use-popover`, `visually-hidden`, `use-icon`, `use-swipe`, `use-media-query`, `use-timers`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/combobox>
Index: [index.md](index.md) lists every other page
Description: A listbox that hangs off a trigger you supply, holding no state of its own.

A listbox that hangs off whatever control you put in it, with the placement, the
arrow keys and the row styling. It holds no state: `use_combobox()` keeps the
open state in your scope, and the selection and closing on an outside click are
yours.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Combobox, ComboboxOption, ComboboxOptionArgs, Options, Text, use_combobox};
use libero::sx::sx;

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    #[option(label = "Dragon fruit")]
    Dragon,
    Elderberry,
    Mango,
    Grape,
}

#[component]
fn Demo() -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);
    // A `combobox` role takes no name from its content, so the label names it.
    let label = format!("{}-label", fruit.id());

    rsx! {
        Combobox {
            state: fruit,
            options: Fruit::options().to_vec(),
            labelled_by: label.clone(),
            select_only: true,
            option_label: move |fruit: Fruit| fruit.label(),
            selected_index: picked().and_then(|fruit| Fruit::options().iter().position(|f| *f == fruit)),
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    selected: picked() == Some(o.value),
                    onpick: move |_| {
                        picked.set(Some(o.value));
                        fruit.close();
                    },
                    "{o.value.label()}"
                }
            },
            Text { id: "{label}", size: "sm", "Fruit" }
            Button {
                variant: "outlined",
                sx: sx().width("280px").max_width("100%"),
                attributes: [
                    fruit.a11y_attributes(),
                    vec![Attribute::new("aria-labelledby", label, None, false)],
                ]
                .concat(),
                onclick: move |_| fruit.toggle(),
                onblur: move |_| fruit.close(),
                match picked() {
                    Some(fruit) => rsx! { "{fruit.label()}" },
                    None => rsx! { "Pick a fruit" },
                }
            }
        }
    }
}
```

## Props

### `Combobox`

| Prop | Type | Default | Description |
|---|---|---|---|
| `state` | `ComboboxState` | required | From `use_combobox()`. The open state, the highlighted row and the id the aria wiring uses. One state drives one combobox. |
| `options` | `OptionSource<T>` | required | The options to list, already filtered: for a suggestion list, by the trigger's text. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. `None::<OptionList<T>>` is pending, for a fetch you drive yourself. `option` runs for every option, and from 200 only the rows in view are drawn, so cap a very large list here. A failed fetch is an empty list, so show your own error beside the field. |
| `option` | `Callback<ComboboxOptionArgs<T>, Element>` | required | Draws one row, usually a `ComboboxOption`. |
| `children` | `Element` | required | The trigger, and anything that belongs with it, such as a hidden input. |
| `empty` | `Element` | - | Shown in place of the list when `options` is empty. |
| `empty_label` | `String` | `combobox.nothing_found` | What screen readers hear when the open list has no options, and what it shows there without `empty`. |
| `loading_label` | `String` | `common.loading` | What screen readers hear while `options` is pending. A pending list shows a `Loader` instead of the rows or `empty`. |
| `labelled_by` | `String` | - | The id of the element that names the list, usually the trigger's label. Screen readers read it with the list. Without it, an `aria-label` on the `Combobox` names the list. |
| `size` | `Size` | `md` | A row's height and font size. |
| `radius` | `ThemeAwareValue` | `sm` | The dropdown's corner radius, or any CSS, e.g. `radius: "0"`. |
| `disabled` | `bool` | `false` | Draws no list and ignores the keys, and the trigger reads as closed. Disable the trigger too. |
| `select_only` | `bool` | `false` | For a trigger with no text to type in, such as a button: Space picks the highlighted row like Enter, and Tab or Alt+Up pick it before closing, as a `Select` does. |
| `selected_index` | `usize` | - | With `select_only`: the row the list opens on, usually the picked one, so Enter keeps the value instead of replacing it with the first row. Unset, or past the end, it opens on the first or last row. |
| `option_label` | `Callback<T, String>` | - | An option's plain text, which `select_only` types ahead on: a typed "b" highlights the next option starting with it and opens the list. Unset, nothing is typed ahead. |

`T` is any `Clone + PartialEq`. `Options` is not required, since you hand the
list in.

`sx`, `class`, `states` and any extra HTML attributes land on the dropdown.

### `ComboboxState`

From `use_combobox()`. It is `Copy`, so it goes into event handlers by value.

| Method | Returns | Description |
|---|---|---|
| `id()` | `String` | The id every part of the wiring is built from. |
| `is_open()` | `bool` | Whether the list is showing. |
| `open()` / `close()` / `toggle()` / `set_open(bool)` | - | Drive it. |
| `active()` | `Option<usize>` | The row the arrow keys are on, indexing `options`. `None` is no highlight. |
| `set_active(Option<usize>)` | - | Move the highlight. |
| `a11y_attributes()` | `Vec<Attribute>` | The trigger's aria wiring, to spread with `attributes:`. |

### `ComboboxOptionArgs<T>`

| Field | Type | Description |
|---|---|---|
| `value` | `T` | The option this row draws. |
| `index` | `usize` | Its position in `options`. |
| `active` | `bool` | Whether the arrow keys are on this row. |
| `disabled` | `bool` | Whether the list refuses this row. |

### `ComboboxOption`

| Prop | Type | Default | Description |
|---|---|---|---|
| `selected` | `bool` | - | Marks the current selection with `aria-selected` and a tint. Leave it unset in a suggestion list. |
| `active` | `bool` | from the `Combobox` | Overrides the keyboard highlight. |
| `disabled` | `bool` | from the `Combobox` | Overrides whether the row is refused. A refused row is greyed and ignores the click and Enter. |
| `onpick` | `EventHandler<()>` | - | Called on a click, and by Enter while the row is active. The only way to pick. |
| `size` | `Size` | the `Combobox`'s | Row height and font size. Defaults to the `Combobox`'s `size`. |
| `radius` | `ThemeAwareValue` | the `Combobox`'s | Corner radius, or any CSS, e.g. `radius: "0"`. Defaults to the `Combobox`'s, reduced so the row nests inside the dropdown. |
| `children` | `Element` | required | The row's content. Put a long label in `span { "data-slot": "label" }` to end it in an ellipsis. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work. Like `sx`, `parts` styles the dropdown, so its parts
are reached even though it is portaled. Rows sit in groups, so the row parts
match at any depth inside the list.

| Part | `data-slot` | Description |
|---|---|---|
| `DropdownPart::Panel` | `dropdown` | The dropdown box itself. |
| `DropdownPart::Listbox` | `listbox` | The scrolling list of rows. |
| `DropdownPart::Group` | `group` | A group of rows that share a label, from an `OptionList`. |
| `DropdownPart::GroupLabel` | `group-label` | A group's heading. |
| `DropdownPart::Option` | `option` | A row. |
| `DropdownPart::OptionLabel` | `label` | A row's `span { "data-slot": "label" }`, which ends in an ellipsis. |
| `DropdownPart::Empty` | `nothing-found` | The text shown when the query matches nothing. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down` | Opens the list, and moves the highlight down. |
| `Up` | Moves the highlight up. With `select_only`, closed: opens the list on the first row. |
| `Alt+Down` | Opens the list without moving the highlight. |
| `Alt+Up` | Open: closes the list. With `select_only`, picks the highlighted row first. |
| `Home` or `End` | Jumps to the first or last row. With `select_only`, closed: opens the list there. |
| `PageUp` or `PageDown` | Moves the highlight 10 rows, stopping at the first or last. |
| `Enter` | Open: picks the highlighted row. Every open starts on the first row, unless a key named another (End: the last; a `select_only` list opens on its `selected_index`), so in a suggestion list Enter replaces the typed text; press `Escape` first to keep it. |
| `Escape` | Open: closes the list and keeps the typed text. Enter then goes to the field, so a form submits. |
| `Letter` | With `select_only` and `option_label`: moves the highlight to the next enabled row starting with the typed text, and opens a closed list. |
| `Space` | With `select_only`, open: picks the highlighted row, as Enter does. |
| `Tab` | Closes the list. With `select_only`, picks the highlighted row first. |

### Libero handles

- Focus stays on your trigger, so typing keeps working.
- An open list with no options says `empty_label`, so an empty search is heard,
  not only seen.
- Android's Back button calls `onopened(false)` while the list is open, rather
  than closing the app.
- From 200 options the list draws only the rows in view. The highlighted row
  stays drawn for `aria-activedescendant`, a `ComboboxOption` row carries
  `aria-posinset` and `aria-setsize` over the whole list, and a grouped row is
  described by its heading.

### You must

- Spread `state.a11y_attributes()` on your trigger, or screen readers cannot
  tie the list to it.
- Close the list on your trigger's blur, or an enclosing `Modal` stops hearing
  Escape while the list stays open.
- On a text field that filters the list, add `aria-autocomplete="list"`:
  `a11y_attributes()` leaves it off, as a button trigger takes none.
- Name the trigger: it becomes a `combobox`, which takes no name from its
  content. Point its `aria-labelledby` at a visible label, as the demos do.
- Pass the same label's id as `labelled_by`, so the list has a name too. A
  `TextField`'s own `label` has the id `{id}-label`, from the `id` you give the field.
- From 200 options, keep every row the theme's `row_height` tall, as a plain
  `ComboboxOption` is: the list places the rows it draws by that height. A
  debug build warns when they differ.

### Example

A search field as your own trigger, with `state.a11y_attributes()` spread on
it and a visible label whose id is `labelled_by`: a screen reader reads a
combobox named by the label, Down moves into the list, and focus never leaves
the field.

## Theme defaults

`ComboboxDefaults` on the theme. Per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |
| `max_dropdown_height` | `&'static str` | Height past which the option list scrolls, `260px`. |
| `sizes` | `Sizes<ComboboxSizeLevel>` | `font_size`, `row_height`, `padding_x` per size. |

`row_height` is a `min-height`, so a taller custom row grows.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-combobox-font-size-<size>` | A row's `font-size` for that size step. |
| `--lsx-combobox-row-height-<size>` | A row's `height` for that size step. |
| `--lsx-combobox-padding-x-<size>` | A row's horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale. A row's corner is 4px
smaller, the dropdown's padding, so it nests inside.

## Data attributes

| Token | Condition | Element |
|---|---|---|
| `size-<size>` | The `size` in effect. | Dropdown, rows |
| `radius-<size>` | The `radius` in effect. | Dropdown, rows |
| `disabled` | `disabled` is set. | Dropdown |
| `active` | The row the arrows are on. | Rows |
| `selected` | `selected` is `true`. | Rows |

Hover tints a row, `selected` tints it more strongly, and `active` adds a ring
on top of either tint.
