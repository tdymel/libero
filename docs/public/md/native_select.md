# NativeSelect

Crate: `libero`
Import: `use libero::components::{Options, NativeSelect};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/native_select.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A styled native `<select>` over an enum, strictly controlled by `value` plus `onchange`.

A styled native select over an enum, with the five slots every field shares:
label, description, the control, helper text, and a validation message.
The options are the enum's variants - `#[derive(Options)]` lists them in
declaration order and names each one - so `onchange` hands back the value itself
rather than a string the caller has to look up again. Strictly controlled:
`value` drives it, and `None` is a real state, the field nobody has filled in
yet.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, NativeSelect};

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
        NativeSelect {
            label: "Size",
            value: value(),
            onchange: move |next| value.set(Some(next)),
        }
    }
}
```

`size` and `radius` step independently, and `disabled` dims the whole field.
Both read the same `FieldDefaults` scale [TextField](text_field.md) does, so the
two line up in one form by construction.

## Captions

`label`, `description` and `helper` are `Caption`s, so each takes either a string
or an `Element`:

```rust,ignore
NativeSelect {
    label: "Plan",
    description: "Billed monthly.",
    helper: rsx! { "Change it any time from " strong { "Settings" } },
    value: plan(),
    onchange: move |next| plan.set(Some(next)),
}
```

A string caption is given an id and named by the select's `aria-describedby`.
Markup is rendered and styled the same way but names nothing - a caller who
passes markup owns its accessibility.

`status` is separate because a validator produces it. `FieldStatus::Error` and
`FieldStatus::Warning` each carry their message; `&str` and `String` convert into
`Error`, so `status: "Pick a size."` works. An error also sets `aria-invalid`; a
warning does not, since it would announce a working field as broken.

Note that `description` and `placeholder` say different things: the description
is a caption above the control, the placeholder is the unpickable first entry
that stands for "nothing picked yet".

## Nothing picked yet

`value` is an `Option`, so a field the user has not filled in is a state the type
can hold rather than a sentinel option in the list. While it is `None` the
`placeholder` shows as the selected entry, disabled and hidden - so the native
control cannot silently take the first option, and once a real value is picked
there is no way back to it. It is dimmed like a text field's placeholder, so an
empty select does not look filled.

```rust
use dioxus::prelude::*;
use libero::components::{Options, NativeSelect};

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
        NativeSelect {
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
required method. `value` is the second, and it defaults to `label` - it is what
each `<option>` posts in a native form, and what `onchange` looks the pick up
by, so no two options may share one. Override it when the label is text a
backend should never receive. The derive overrides it already: for an
enum the wire value is the variant's name, never a customised label.

```rust
use dioxus::prelude::*;
use libero::components::{Options, NativeSelect};

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
        NativeSelect {
            label: "Order",
            placeholder: "Pick an order",
            value: selected(),
            options: orders.clone(),
            onchange: move |order| selected.set(Some(order)),
        }
    }
}
#
# type OrderId = u32;
```

`onchange` hands back the `Order`, not an id to look up again. Note that
`PartialEq` is how the component finds the selected option, so it must be
**identity, not content**: an `Order` that derives `PartialEq` over every field
loses its selection the moment one of those fields changes underneath it.

Labels that need data the value does not carry go through `option_label`, which
runs during render - so it can read a lookup table or a locale from context:

```rust,ignore
NativeSelect {
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

## Disabled and grouped options

`options` takes the same `OptionList` as [Select](select.md). A disabled
`OptionItem` renders as `<option disabled>`: shown and announced, but the picker
and the arrow keys pass over it. A named group renders as an `<optgroup>`.

```rust
use dioxus::prelude::*;
use libero::components::{NativeSelect, OptionItem, OptionList, Options};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Pear,
    Cherry,
    Plum,
}

#[component]
fn Demo() -> Element {
    let mut fruit = use_signal(|| None::<Fruit>);

    rsx! {
        NativeSelect {
            label: "Fruit",
            placeholder: "Pick a fruit",
            value: fruit(),
            options: OptionList::grouped()
                .group("Pome", [Fruit::Apple, Fruit::Pear])
                .group("Stone", [OptionItem::new(Fruit::Cherry).disabled(true), Fruit::Plum.into()]),
            onchange: move |next| fruit.set(Some(next)),
        }
    }
}
```

`OptionList::from_options().disabling(..)` keeps the enum's own list and refuses
a few of it.

## Accessibility

Leave `label` unset only when something else already names the select; a bare
`<select>` with no accessible name is a defect. A caller-supplied
`aria-describedby` joins the one built from the caption slots - the caller's
ids first - so the validation message is never lost.

## Props

### `NativeSelect`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<T>` | - | The selected option; strictly controlled. `None` shows `placeholder` and selects nothing. |
| `onchange` | `EventHandler<T>` | - | Called with the option the caller should select next. Never fires for the placeholder, which cannot be picked. A handler that ignores the pick leaves it in the DOM: a `<select>`'s change cannot be cancelled. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set - `String`s, or records fetched from a server - passes them here, since only an enum lists its own. A `Vec<T>` converts; an `OptionList` adds disabled options and named groups. A pending source draws no options. |
| `option_label` | `Callback<T, String>` | `T::label()` | Overrides what the derive named an option. Returns a `String`, not an `OptionLabel`: an `<option>` holds text and nothing else. |
| `placeholder` | `String` | - | Shown while `value` is `None`, as an unpickable first entry. |
| `label` | `Caption` | - | The field's caption, above the control. Names the field through a `for`/`id` pair. |
| `description` | `Caption` | - | Between the label and the control: what to pick. |
| `helper` | `Caption` | - | Under the control: constraints, or what the choice affects. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. No native `required`, so an untouched select is not announced invalid; `validate` or the surrounding `Form` enforces it. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. There is no `readonly`: a native `<select>` has no read-only state. For a picker that stays focusable and posted but cannot change, use `Select`. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field. `NativeSelectDefaults`
keeps only what is genuinely this component's: which `size` and `radius` it
starts at.

| Field | Type | Description |
|---|---|---|
| `field.gap` | `&'static str` | Vertical gap between the slots. |
| `field.frame_gap` | `&'static str` | Horizontal gap inside the frame. |
| `field.sizes` | `Sizes<FieldSizeLevel>` | `label_font_size`, `caption_font_size`, `font_size`, `height`, `padding_y`, `padding_x` per size. |
| `native_select.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `native_select.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-field-gap` | Vertical gap between the slots. |
| `--lsx-field-label-font-size-<size>` | The label's `font-size` for that size step. |
| `--lsx-field-caption-font-size-<size>` | `font-size` of the description, helper and status text. |
| `--lsx-field-frame-gap` | Horizontal gap inside the frame. |
| `--lsx-field-font-size-<size>` | `font-size` of the control for that size step. |
| `--lsx-field-height-<size>` | The frame's `min-height` for that size step. |
| `--lsx-field-padding-y-<size>` | The frame's vertical padding for that size step. |
| `--lsx-field-padding-x-<size>` | The frame's horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its own.

## Data attributes

State tokens on the wrapper's and the frame's `data-state`, space separated. The
`<select>` itself carries only `data-placeholder` while `value` is `None`, which
dims it.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `disabled` | `disabled` is set. |
| `required` | `required` is set. |
| `warning` | `status` is a `Warning`. |
| `error` | `status` is an `Error`. |

The caption slots carry `data-slot="description"`, `"helper"`, `"status"` and
`"required"`, which is how the wrapper styles them.
