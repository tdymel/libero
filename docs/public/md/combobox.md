# Combobox

Crate: `libero`
Import: `use libero::components::{Combobox, ComboboxOption, ComboboxOptionArgs, use_combobox};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/combobox>
Index: [index.md](index.md) lists every other page
Description: A listbox that hangs off a trigger you supply, holding no state of its own.

A listbox that hangs off whatever control you put in it. It holds no state of
its own. `use_combobox()` keeps the open state in your scope, the selection is
yours, `option` draws the rows and `children` is the trigger. The combobox adds
the placement, the arrow keys and the row styling. Closing on an outside click
is yours, and `onpick` is the only way to pick.

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

/// The fruit whose label contains `query`, case-insensitively.
fn matching(query: &str) -> Vec<Fruit> {
    let query = query.to_lowercase();
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
| `loading_label` | `String` | `common.loading` | What screen readers hear while `options` is pending. A pending list shows a `Loader` instead of the rows or `empty`. |
| `size` | `Size` | `md` | A row's height and font size. |
| `radius` | `Size` | `sm` | The dropdown's corner radius. |
| `disabled` | `bool` | `false` | Blocks the arrow keys. Disable the trigger too. |

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
| `onpick` | `EventHandler<()>` | - | A click, or Enter while the row is active. |
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
| `ComboboxPart::Listbox` | `listbox` | The scrolling list of rows. |
| `ComboboxPart::Group` | `group` | A group of rows that share a label, from an `OptionList`. |
| `ComboboxPart::GroupLabel` | `group-label` | A group's heading. |
| `ComboboxPart::Option` | `option` | A `ComboboxOption` row. |
| `ComboboxPart::OptionLabel` | `label` | A row's `span { "data-slot": "label" }`, which ends in an ellipsis. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down` | Opens the list, and moves the highlight down. |
| `Up` | Moves the highlight up. |
| `Home` or `End` | Jumps to the first or last row. |
| `Enter` | Picks the highlighted row. |
| `Escape` or `Tab` | Close the list. |

### Libero handles

- Focus stays on your trigger, so typing keeps working.

### You must

- Spread `state.a11y_attributes()` on your trigger, or screen readers cannot
  tie the list to it.
- Close the list on your trigger's blur, or an enclosing `Modal` stops hearing
  Escape while the list stays open.
- Name the trigger: it becomes a `combobox`, which takes no name from its
  content. Point a button trigger's `aria-labelledby` at a visible label, and
  give a text field a `label`.

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
