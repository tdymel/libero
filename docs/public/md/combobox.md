# Combobox

Crate: `libero`
Import: `use libero::components::{Combobox, ComboboxOption, ComboboxOptionArgs, use_combobox};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/combobox>
Index: [index.md](index.md) - every other component's markdown page
Description: A listbox that hangs off a trigger you supply, holding no state of its own.

A listbox that hangs off whatever control you put in it. It holds no state of
its own. `use_combobox()` keeps the open state in your scope, the selection is
yours, `option` draws the rows and `children` is the trigger. The combobox adds
the placement, the arrow keys and the row styling.

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

## State

There is no `value`, no `onchange` and no open state inside. The list is
portaled and placed under the children, or above them when there is no room
below. `use_combobox()` returns a `ComboboxState`, which you pass as `state` and
can drive yourself:

```rust,ignore
let fruit = use_combobox();

fruit.is_open();           // is the list showing
fruit.open();             // .close(), .toggle(), .set_open(bool)
fruit.active();           // the row the arrow keys are on
fruit.a11y_attributes();  // the aria wiring for whatever control you use
```

The combobox writes the same state when a key asks it to. ArrowDown opens, and
Enter, Escape and Tab close. Closing on an outside click and picking are yours.
`ComboboxOption`'s `onpick` is the only way to pick, on a click and on Enter
alike. An open list with no options and no `empty` renders nothing.

`children` is the trigger, and anything that has to travel with it, such as a
hidden `input` for a plain form post:

```rust,ignore
Combobox {
    // ..
    TextField { value: text(), oninput: move |next| text.set(next) }
    input { r#type: "hidden", name: "fruit", value: "{text()}" }
}
```

The arrow keys work whatever the trigger is.

## Rows

`option` draws one row and gets the value and its index. `ComboboxOption` is the
styled row. It takes the click and reads its id and highlight from the
`Combobox` around it, so it needs no wiring props. `args.active` is there for a
row drawn without `ComboboxOption`.

Bare text that is too long is cut at the row's edge. Put it in
`span { "data-slot": "label", .. }` to end it in an ellipsis. `selected` sets
`aria-selected` and tints the row, so leave it out in a suggestion list. A
taller custom row grows.

Every option renders, so keep the list short. `options` is where you cap it. The
list scrolls past the theme's `max_dropdown_height`, `260px`.

## Searching

There is no `filter` prop and no search field. Filter the `Vec` yourself with
the trigger's own value:

```rust,ignore
let suggestions = use_combobox();

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
                onpick: move |_| { text.set(o.value.label()); suggestions.close(); },
                "{o.value.label()}"
            }
        },
        TextField {
            attributes: suggestions.a11y_attributes(),
            value: text(),
            oninput: move |next| { text.set(next); suggestions.open(); },
        }
    }
}
```

Groups and disabled options come through `options`, as an `OptionList<T>`:

```rust,ignore
OptionList::grouped()
    .group("Orchard", [Fruit::Apple.into(), OptionItem::new(Fruit::Cherry).disabled(true)])
    .group("Tropical", [Fruit::Banana, Fruit::Mango].map(OptionItem::new))
```

Each group is a `role="group"` named by its heading. Your order is kept, so a
group label used again after another group draws its heading again. A disabled
row is read out, and the arrows and clicks skip it.

## Fetching options

The shortest async list is a `use_resource` passed to `options`:

```rust,ignore
let fruit = use_resource(move || async move { search(query()).await });

rsx! {
    Combobox { state: suggestions, options: fruit, .. }
}
```

While the source is pending, a [Loader](loader.md) replaces the rows and
`empty`, and screen readers hear `loading_label`. So an async list does not
flash "no results" between a keystroke and its answer. A failed fetch is an
empty list, so show your own error beside the field.

When you drive the request yourself, `None` is pending and `Some(list)` is the
answer:

```rust,ignore
/// How long the fake search takes.
const LATENCY: Duration = Duration::from_millis(700);

/// Stands in for the server: the fruit whose label contains `query`.
fn matching(query: &str) -> Vec<Fruit> {
    let query = query.to_lowercase();
    Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| fruit.label().to_lowercase().contains(&query))
        .collect()
}

let suggestions = use_combobox();
let mut text = use_signal(String::new);
// One signal, not a list plus a `loading` flag: `None` *is* the search in
// flight, so the two can never disagree.
let mut results = use_signal(|| Some(Vec::<Fruit>::new()));
// The answer still on its way. Replacing it drops the older one, so a slow
// answer never overwrites a newer query's.
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
                onpick: move |_| { text.set(o.value.label()); suggestions.close(); },
                "{o.value.label()}"
            }
        },
        TextField {
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
```

There is no creatable mode yet, and one `ComboboxState` drives one `Combobox`.

## Accessibility

Focus stays on your trigger, so typing keeps working. Spread
`state.a11y_attributes()` on it, or screen readers cannot tie the list to it:

```rust,ignore
TextField {
    attributes: suggestions.a11y_attributes(),
    value: text(),
    oninput: move |next| { text.set(next); suggestions.open(); },
}
```

ArrowDown opens and moves down, ArrowUp moves up, Home and End jump to the ends,
Enter picks and closes, and Escape and Tab close. Disabled rows are skipped.
Close the list on your trigger's blur (`onblur: move |_| suggestions.close()`),
or an enclosing `Modal` or `HoverCard` stops hearing Escape while the list stays
open.

## Props

### `Combobox`

| Prop | Type | Default | Description |
|---|---|---|---|
| `state` | `ComboboxState` | required | From `use_combobox()`. The open state, the highlighted row and the id the aria wiring uses. |
| `options` | `OptionSource<T>` | required | The options to list, already filtered. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. `None::<OptionList<T>>` is pending, for a fetch you drive yourself. |
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
