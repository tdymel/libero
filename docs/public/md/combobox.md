# Combobox

Crate: `libero`
Import: `use libero::components::{Combobox, ComboboxOption, ComboboxOptionArgs, use_combobox};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/combobox>
Index: [index.md](index.md) - every other component's markdown page
Description: A listbox that hangs off a caller-supplied trigger, holding no state of its own.

A listbox that hangs off whatever control you put in it. It holds no state of
its own: `use_combobox()` keeps it in the caller's scope, the selection is the
caller's entirely, the rows are drawn by `option`, and the trigger is just
`children`. All it adds is the placement, the arrow keys, and the row theming.

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

## It holds nothing

There is no `value`, no `onchange`, and no open state inside. `Combobox` is an
arrangement, not a control: a relative wrapper, your `children` in it, and an
absolutely positioned list under them.

What open state there is lives in **your** scope. `use_combobox()` returns a
`ComboboxState` - three signals and the id that ties them together - which you
pass in as `state` and can drive yourself at any time:

```rust
let fruit = use_combobox();

fruit.opened();           // is the list showing
fruit.open();             // .close(), .toggle(), .set_opened(bool)
fruit.active();           // the row the arrow keys are on
fruit.a11y_attributes();  // the aria wiring for whatever control you use
```

`Combobox` writes those same signals when a key asks it to - ArrowDown opens,
Enter/Escape/Tab close - but it is your signal either way, so there is no
"controlled or uncontrolled" question to answer and no change handler to route.

The rest stays yours:

- **Closing on an outside click.** `Combobox` never takes focus, so it has
  nothing to blur; call `close()` from your trigger's own blur if you want it.
- **Picking.** `ComboboxOption`'s `onpick` is the only path, on a click and on
  Enter alike, and what it does with the value is entirely up to you.

An open list with no options and no `empty` renders nothing at all, so
`options` being empty is a perfectly good way to keep the dropdown away - you do
not have to gate `opened` on it.

## Children, not a target

`children` is the trigger, and anything that has to travel with it - a hidden
`input` for a plain form post, say:

```rust
Combobox {
    // ..
    TextField { value: text(), onchange: move |next| text.set(next) }
    input { r#type: "hidden", name: "fruit", value: "{text()}" }
}
```

Keyboard events are caught on the wrapper, where they bubble to, rather than on
a field `Combobox` owns - so the arrow keys work whatever the trigger is, and
`Combobox` never has to hand handlers back out.

## Rows

`option` draws one row and is handed the value and its index. `ComboboxOption`
is the themed row: it takes the click, registers itself as Enter's target while
it is active, cancels `mousedown` so a click does not blur the trigger out from
under itself, and reads its own `id` and highlight from the `Combobox` around it
- so a row needs no wiring props at all. `args.active` is there for a row drawn
without `ComboboxOption`.

`selected` is deliberately optional. It sets `aria-selected` and tints the row -
which is select semantics. A suggestion list has no selection, so it just does
not pass it.

A row's height is a `min-height`, so a taller custom row simply grows. Nothing
has to be told about it: every row is a real element, rendered eagerly.

## How long a list

Every option renders, eagerly: `Combobox` draws the rows and hands them down as
a `Vec<Element>` rather than a callback the list could invoke lazily. That is
deliberate. Two `Callback`s built in the same scope on different renders compare
*equal* - `GenerationalBox::ptr_eq` sees the recycled slot - so a lazy row
callback lets the whole subtree memoize, and a filtered `options` leaves stale
rows on screen. A `Vec<Element>` never compares equal, which is the guarantee
the rows need.

The list scrolls past `260px`, from the theme's `max_dropdown_height`. That is
not a prop yet - deliberately. **Keeping the list short enough to render is the
caller's job**, and it is the job `options` already exists for: hand in the
matches, capped however the data wants capping.

## Searching

There is no `filter` prop and no search field. Narrowing a list is filtering a
`Vec`, and the caller already has the query - it is the trigger's own value:

```rust
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
            onchange: move |next| { text.set(next); suggestions.open(); },
        }
    }
}
```

That also covers a filter on something other than the label - an option shown by
name but searched by email is a different closure in the same `filter` call, not
a prop.

## Fetching options

When the options come from a request, set `loading` while it runs. The dropdown
then shows a [`Loader`](loader.md) in place of the rows **and of `empty`** - an
async list's `options` is empty between a keystroke and its answer, and without
`loading` every keystroke would flash the `empty` content first. The rows are
replaced too: they belong to the previous query, and the arrows and Enter skip
them.

```rust
let suggestions = use_combobox();
let mut text = use_signal(String::new);
let mut results = use_signal(Vec::<Fruit>::new);
let mut loading = use_signal(|| false);

rsx! {
    Combobox {
        state: suggestions,
        options: results(),
        loading: loading(),
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
                loading.set(true);
                spawn(async move {
                    results.set(search(&next).await);
                    loading.set(false);
                });
            },
        }
    }
}
```

## What it does not do yet

Multi-selection is nothing but a longer list of `selected` rows, so it needs no
support here, but there is no creatable mode and no option groups. One `ComboboxState` drives one `Combobox`. `max_dropdown_height` is not a prop yet. The dropdown is
positioned with plain absolute placement below the wrapper - it is not portaled
and it does not flip when it runs out of room below.

## Accessibility

Focus never moves. It stays wherever you put it - in a text field, on a button -
and the arrows move a *highlight* instead, which is what lets you keep typing
while arrowing through the suggestions. That is the `aria-activedescendant`
pattern, and it is the whole reason the state is a handle rather than a pile of
props: the attribute belongs on the focused trigger, and the trigger is the one
element `Combobox` does not render.

`state.a11y_attributes()` is that wiring, ready to spread:

```rust
TextField {
    attributes: suggestions.a11y_attributes(),
    value: text(),
    onchange: move |next| { text.set(next); suggestions.open(); },
}
```

It sets `role="combobox"`, `aria-haspopup="listbox"`, `aria-expanded`,
`aria-controls`, and `aria-activedescendant` while the list is open. The ids
come from the handle, so nothing has to line up by hand:

| Element | `id` | Set by |
|---|---|---|
| The list | `<state id>-listbox` | `Combobox` |
| A row | `<state id>-option-<index>` | `ComboboxOption` |

The list is `role="listbox"` and rows are `role="option"`, with `aria-selected`
only on rows that pass `selected` - a suggestion list has no selection to
announce.

Keyboard, from anywhere inside the wrapper: ArrowDown opens and moves down,
ArrowUp moves up, Home and End jump to the ends, Enter picks the active row and
closes, Escape and Tab close. `disabled` blocks all of it.

While `loading`, the loader is the only content of the dropdown, so it is the
one that speaks: it renders `role="status"` with `loading_label` as a visually
hidden text node, and the dropdown carries `aria-busy="true"`.

## Props

### `Combobox`

| Prop | Type | Default | Description |
|---|---|---|---|
| `state` | `ComboboxState` | - | From `use_combobox()`. Required. |
| `options` | `Vec<T>` | - | The options to list, already filtered. Required. |
| `option` | `Callback<ComboboxOptionArgs<T>, Element>` | - | Draws one row. Required. |
| `children` | `Element` | - | The trigger, and anything else that belongs with it. |
| `empty` | `Element` | - | Shown in place of the list when `options` is empty. |
| `loading` | `bool` | `false` | The options are being fetched: a labelled `Loader` replaces the rows and `empty`, and the dropdown is `aria-busy`. |
| `loading_label` | `String` | theme | What the loader announces while `loading`. Unset, `theme.combobox.labels.loading` - "Loading" in `ComboboxLabels::ENGLISH`. |
| `size` | `Size` | `md` | A row's height and font size. |
| `radius` | `Size` | `sm` | The dropdown's corner radius. |
| `disabled` | `bool` | `false` | Blocks the arrow keys. |

`T` is any `Clone + PartialEq` - `Options` is not required, since the list is
handed in rather than derived.

`sx`, `class`, `states` and any extra HTML attributes land on the **dropdown**,
not on a wrapper: the element the dropdown hangs off is positioning scaffolding
rather than something to style.

### `ComboboxState`

From `use_combobox()`, which lives with the component rather than in `hooks` -
it is `Combobox`'s half of the contract, not a general-purpose hook. `Copy`, so
it goes into event handlers by value.

| Method | Returns | Description |
|---|---|---|
| `id()` | `String` | The id every part of the wiring is built from. |
| `opened()` | `bool` | Whether the list is showing. |
| `open()` / `close()` / `toggle()` / `set_opened(bool)` | - | Drive it. |
| `active()` | `usize` | The row the arrow keys are on, indexing `options`. |
| `set_active(usize)` | - | Move the highlight. |
| `a11y_attributes()` | `Vec<Attribute>` | The trigger's aria wiring, to spread with `attributes:`. |

### `ComboboxOptionArgs<T>`

| Field | Type | Description |
|---|---|---|
| `value` | `T` | The option this row draws. |
| `index` | `usize` | Its position in `options`. |
| `active` | `bool` | Whether the arrow keys are on this row. |

### `ComboboxOption`

| Prop | Type | Default | Description |
|---|---|---|---|
| `selected` | `bool` | - | The current selection - `aria-selected` and a tint. Unset for a suggestion list. |
| `active` | `bool` | from the `Combobox` | Overrides the keyboard highlight. |
| `onpick` | `EventHandler<()>` | - | A click, or Enter while the row is active. |
| `size` | `Size` | the `Combobox`'s | Row height and font size. |
| `radius` | `Size` | the `Combobox`'s | Corner radius, tightened by the dropdown's padding. |

## Theme defaults

`ComboboxDefaults` on the theme; per-size values live in its `sizes` scale,
seeded with the numbers `FieldDefaults` uses.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |
| `max_dropdown_height` | `&'static str` | Height past which the option list scrolls; `260px`. No prop overrides it yet. |
| `sizes` | `Sizes<ComboboxSizeLevel>` | `font_size`, `row_height`, `padding_x` per size. |

`row_height` is applied as a `min-height`, so a taller custom row grows rather
than being clipped.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-combobox-font-size-<size>` | A row's `font-size` for that size step. |
| `--lsx-combobox-row-height-<size>` | A row's `height` for that size step. |
| `--lsx-combobox-padding-x-<size>` | A row's horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its
own. A row's corner is `max(0px, calc(<that> - 4px))`, the 4px being the
dropdown's padding: a row nests that far inside, so an equal radius would cross
the dropdown's own corner. The dropdown also clips, so nothing can escape it at
`xxl`, where the radius is 64px.

## Data attributes

| Token | Condition | Element |
|---|---|---|
| `size-<size>` | The `size` in effect. | Dropdown, rows |
| `radius-<size>` | The `radius` in effect. | Dropdown, rows |
| `disabled` | `disabled` is set. | Dropdown |
| `active` | The row the arrows are on. | Rows |
| `selected` | `selected` is `true`. | Rows |

A row draws three states without them cancelling out: hover tints it, `selected`
tints it more strongly, and `active` adds a focus ring *on top of* whichever
tint is underneath. A tint alone cannot mark the keyboard's row when that row is
already tinted by the selection, which is why the highlight is a ring.
