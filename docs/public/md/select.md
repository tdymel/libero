# Select

Crate: `libero`
Import: `use libero::components::{Options, Select};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/select>
Index: [index.md](index.md) - every other component's markdown page
Description: A styled native `<select>` over an enum, strictly controlled by `value` plus `onchange`.

A styled native select over an enum, always wrapped in its own `label` element.
The options are the enum's variants - `#[derive(Options)]` lists them in
declaration order and names each one - so `onchange` hands back the value itself
rather than a string the caller has to look up again. Strictly controlled:
`value` drives it, and `None` is a real state, the field nobody has filled in
yet.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, Select};

#[derive(Clone, Copy, PartialEq, Options)]
enum FontSize {
    #[option(label = "Extra small")]
    Xs,
    Small,
    Medium,
    Large,
    #[option(label = "Extra large")]
    Xl,
}

#[component]
fn Demo() -> Element {
    let mut value = use_signal(|| Some(FontSize::Small));

    rsx! {
        Select {
            label: "Size",
            value: value(),
            onchange: move |next| value.set(Some(next)),
        }
    }
}
```

`size` and `radius` step independently, and `disabled` dims the whole control.

## Nothing picked yet

`value` is an `Option`, so a field the user has not filled in is a state the type
can hold rather than a sentinel option in the list. While it is `None` the
`placeholder` shows as the selected entry, disabled and hidden - so the native
control cannot silently take the first option, and once a real value is picked
there is no way back to it.

```rust
use dioxus::prelude::*;
use libero::components::{Options, Select};

#[derive(Clone, Copy, PartialEq, Options)]
enum FontSize {
    Small,
    Medium,
    Large,
}

#[component]
fn Demo() -> Element {
    let mut size = use_signal(|| None::<FontSize>);

    rsx! {
        Select {
            label: "Size",
            placeholder: "Pick a size",
            value: size(),
            onchange: move |next| size.set(Some(next)),
        }
    }
}
```

## A runtime set of options

`options` narrows or reorders the list. Only an enum lists its own options, so a
set that is data - `String`s, or records fetched from a server - passes them
here. Any type can be an option by implementing `Options`; `label` is the only
required method.

```rust
use dioxus::prelude::*;
use libero::components::{Options, Select};

#[derive(Clone, PartialEq)]
struct Order {
    id: OrderId,
    customer: String,
}

impl Options for Order {
    fn label(&self) -> String {
        format!("#{} - {}", self.id, self.customer)
    }
}

#[component]
fn Demo(orders: Vec<Order>) -> Element {
    let mut selected = use_signal(|| None::<Order>);

    rsx! {
        Select {
            label: "Order",
            placeholder: "Pick an order",
            value: selected(),
            options: orders.clone(),
            onchange: move |order| selected.set(Some(order)),
        }
    }
}
```

`onchange` hands back the `Order`, not an id to look up again. Note that
`PartialEq` is how the component finds the selected option, so it must be
**identity, not content**: an `Order` that derives `PartialEq` over every field
loses its selection the moment one of those fields changes underneath it.

Labels that need data the value does not carry go through `option_label`, which
runs during render - so it can read a lookup table or a locale from context:

```rust
Select {
    value: selected(),
    options: order_ids(),
    option_label: move |id: OrderId| orders.read().title_of(id),
    onchange: move |id| selected.set(Some(id)),
}
```

An `<option>` holds text and nothing else, so `option_label` returns a `String`.
There is no rich form here the way [Tabs](tabs.md) and
[SegmentedControl](segmented_control.md) have `OptionLabel::rich` - an icon per
option needs a listbox rather than a native `<select>`.

## Accessibility

The root is always a `<label>` wrapping the `<select>`, so a click anywhere on the
control focuses it and the label text - the `<span>` `label` renders - names the
select without an `id`/`for` pair. Leave `label` unset only when something else
already names the select; a bare `<select>` with no accessible name is a defect.
`disabled` sets the native `disabled` attribute, so the browser handles the
focus and interaction semantics.

The selection is written as `selected` on each `<option>`, not as `value` on the
`<select>`. The property lands on the option itself, so it does not depend on
the parent's children already existing - which is what the old `value` path got
wrong on the creating render, and it means server-rendered HTML carries the
right selection instead of none.

One gap remains, and it is the same class as any controlled native input: if a
caller ignores an `onchange`, the vdom is unchanged, the differ writes nothing,
and the DOM keeps the user's pick. A `<select>`'s change is not activation
behaviour, so the cancel-the-click trick that fixes checkboxes and radios does
not apply here.

## Props

### `Select`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<T>` | - | The selected option; strictly controlled. `None` shows `placeholder` and selects nothing. |
| `onchange` | `EventHandler<T>` | - | Called with the option the caller should select next. Never fires for the placeholder, which cannot be picked. |
| `options` | `Vec<T>` | `T::options()` | Narrows or reorders the list. A runtime set - `String`s, or records fetched from a server - passes them here, since only an enum lists its own. |
| `option_label` | `Callback<T, String>` | `T::label()` | Overrides what the derive named an option. Returns a `String`, not an `OptionLabel`: an `<option>` holds text and nothing else. |
| `placeholder` | `String` | - | Shown while `value` is `None`, as an unpickable first entry. |
| `disabled` | `bool` | `false` | Disables interaction and dims the select. |
| `label` | `String` | - | The field's own caption, above the control. The wrapper is a `<label>` either way; unset just leaves it wordless. |
| `label_sx` | `Sx` | - | Styles the caption alone - the rest of `sx` lands on the wrapper. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`SelectDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |
| `sizes` | `Sizes<SelectSizeLevel>` | `font_size`, `height`, `padding_x` per size. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-select-font-size-<size>` | `font-size` for that size step. |
| `--lsx-select-height-<size>` | `height` for that size step. |
| `--lsx-select-padding-x-<size>` | Horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its own.

## Data attributes

State tokens on the `<select>`'s `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `disabled` | `disabled` is set. |
