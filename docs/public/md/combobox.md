# Combobox

Crate: `libero`
Import: `use libero::components::{Combobox, ComboboxOption, ComboboxOptionArgs, use_combobox};`
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
use libero::components::{
    Button, Combobox, ComboboxOption, ComboboxOptionArgs, Options, use_combobox,
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[component]
fn Demo() -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);

    rsx! {
        Combobox {
            state: fruit,
            options: Fruit::options().to_vec(),
            // A button has no text to type in: Space and Tab pick, as in a `Select`.
            select_only: true,
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
            Button {
                variant: "outlined",
                attributes: fruit.a11y_attributes(),
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

A suggestion list filters the options by the trigger's text, marks no row as
selected, and carries a hidden input for a plain form post:

```rust
use dioxus::prelude::*;
use libero::components::{
    Combobox, ComboboxOption, ComboboxOptionArgs, Options, TextField, use_combobox,
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[component]
fn Demo() -> Element {
    let suggestions = use_combobox();
    let mut text = use_signal(String::new);

    let matches: Vec<Fruit> = Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| fruit.label().to_lowercase().contains(&text().to_lowercase()))
        .collect();

    rsx! {
        Combobox {
            state: suggestions,
            options: matches,
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        text.set(o.value.label());
                        suggestions.close();
                    },
                    "{o.value.label()}"
                }
            },
            TextField {
                placeholder: "Type a fruit",
                value: text(),
                attributes: suggestions.a11y_attributes(),
                onblur: move |_| suggestions.close(),
                oninput: move |next| {
                    text.set(next);
                    suggestions.open();
                },
            }
            input { r#type: "hidden", name: "fruit", value: "{text()}" }
        }
    }
}
```

A search you drive yourself passes `None` while it runs, which shows the
loader instead of `empty`. A `use_resource` passed to `options` does the same.

```rust
use std::time::Duration;

use dioxus::prelude::*;
use libero::{
    components::{
        Combobox, ComboboxOption, ComboboxOptionArgs, OptionList, Options, Text, TextField,
        use_combobox,
    },
    platform::{TimerSubscription, timer},
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

/// How long the fake search takes.
const LATENCY: Duration = Duration::from_millis(700);

/// The fruit whose label contains `query`, case-insensitively and ignoring surrounding space.
fn matching(query: &str) -> Vec<Fruit> {
    let query = query.trim().to_lowercase();
    Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| fruit.label().to_lowercase().contains(&query))
        .collect()
}

#[component]
fn Demo() -> Element {
    let suggestions = use_combobox();
    let mut text = use_signal(String::new);
    // `None` is the search in flight.
    let mut results = use_signal(|| Some(Vec::<Fruit>::new()));
    // Replacing the pending answer drops it, so a slow answer never overwrites a newer one.
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));

    rsx! {
        Combobox {
            state: suggestions,
            options: results().map(OptionList::from),
            loading_label: "Searching fruit",
            empty_label: "No fruit matches",
            empty: rsx! { Text { size: "sm", "No fruit matches" } },
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        text.set(o.value.label());
                        suggestions.close();
                    },
                    "{o.value.label()}"
                }
            },
            TextField {
                placeholder: "Type a fruit",
                value: text(),
                attributes: suggestions.a11y_attributes(),
                onblur: move |_| suggestions.close(),
                oninput: move |next: String| {
                    text.set(next.clone());
                    suggestions.open();
                    results.set(None);
                    let answer = timer().map(|timer| {
                        timer.after(LATENCY, Box::new(move || results.set(Some(matching(&next)))))
                    });
                    pending.set(answer);
                },
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
| `options` | `OptionSource<T>` | required | The options to list, already filtered. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. `None::<OptionList<T>>` is pending, for a fetch you drive yourself. Every option renders, so cap the list here. A failed fetch is an empty list, so show your own error beside the field. |
| `option` | `Callback<ComboboxOptionArgs<T>, Element>` | required | Draws one row, usually a `ComboboxOption`. |
| `children` | `Element` | required | The trigger, and anything that belongs with it, such as a hidden input. |
| `empty` | `Element` | - | Shown in place of the list when `options` is empty. |
| `empty_label` | `String` | `combobox.nothing_found` | What screen readers hear when the open list has no options, and what it shows there without `empty`. |
| `loading_label` | `String` | `common.loading` | What screen readers hear while `options` is pending. A pending list shows a `Loader` instead of the rows or `empty`. |
| `labelled_by` | `String` | - | The id of the element that names the list, usually the trigger's label. Screen readers read it with the list. |
| `size` | `Size` | `md` | A row's height and font size. |
| `radius` | `Size` | `sm` | The dropdown's corner radius. |
| `disabled` | `bool` | `false` | Draws no list and ignores the keys, and the trigger reads as closed. Disable the trigger too. |
| `select_only` | `bool` | `false` | For a trigger with no text to type in, such as a button: Space picks the highlighted row like Enter, and Tab or Alt+Up pick it before closing, as a `Select` does. |

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
| `onpick` | `EventHandler<()>` | - | A click, or Enter while the row is active. The only way to pick. |
| `size` | `Size` | the `Combobox`'s | Row height and font size. |
| `radius` | `Size` | the `Combobox`'s | Corner radius, reduced so the row nests inside the dropdown. |
| `children` | `Element` | required | The row's content. |

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
| `Up` | Moves the highlight up. |
| `Home` or `End` | Jumps to the first or last row. |
| `PageUp` or `PageDown` | Moves the highlight 10 rows, stopping at the first or last. |
| `Enter` | Open: picks the highlighted row. Every open starts on the first row, so in a suggestion list Enter replaces the typed text; press `Escape` first to keep it. |
| `Escape` | Open: closes the list and keeps the typed text. Enter then goes to the field, so a form submits. |
| `Space` | With `select_only`, open: picks the highlighted row, as Enter does. |
| `Tab` | Closes the list. With `select_only`, picks the highlighted row first. |

### Libero handles

- Focus stays on your trigger, so typing keeps working.
- An open list with no options says `empty_label`, so an empty search is heard,
  not only seen.
- Android's Back button calls `onopened(false)` while the list is open, rather
  than closing the app.

### You must

- Spread `state.a11y_attributes()` on your trigger, or screen readers cannot
  tie the list to it.
- Close the list on your trigger's blur, or an enclosing `Modal` stops hearing
  Escape while the list stays open.
- Name the trigger: it becomes a `combobox`, which takes no name from its
  content. Point a button trigger's `aria-labelledby` at a visible label, and
  give a text field a `label`.
- Pass the same label's id as `labelled_by`, so the list has a name too.

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
