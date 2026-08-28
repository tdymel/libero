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

```rust
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
there is no way back to it.

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
required method.

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
```

`onchange` hands back the `Order`, not an id to look up again. Note that
`PartialEq` is how the component finds the selected option, so it must be
**identity, not content**: an `Order` that derives `PartialEq` over every field
loses its selection the moment one of those fields changes underneath it.

Labels that need data the value does not carry go through `option_label`, which
runs during render - so it can read a lookup table or a locale from context:

```rust
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

## Accessibility

Focus lands on the `<select>`, but the focus ring is drawn by the frame around
it, keyed off `:has(:focus-visible)` - so a keyboard focus rings the whole field
and a mouse click does not ring anything.

The label is a real `<label for>` paired with the select's `id`. It was a
wrapping `<label>` before the field port; the pair replaced it because a wrapping
label swallows clicks on anything else inside the field. A caller-supplied `id`
takes over the generated one, and every slot follows it.

Whichever of the description, helper and status slots are filled are joined into
the select's `aria-describedby`, in reading order. A caller who passes their own
`aria-describedby` wins outright: theirs is used and the generated list is
dropped.

`required` sets the native attribute and `aria-required`, and adds an asterisk to
the label. The asterisk is `aria-hidden` - `aria-required` already carries it to
assistive technology.

Leave `label` unset only when something else already names the select; a bare
`<select>` with no accessible name is a defect. `disabled` sets the native
`disabled` attribute, so the browser handles the focus and interaction semantics.

The chevron is the browser's own: drawing ours would mean `appearance: none`, and
with it the native picker affordance on every platform.

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

### `NativeSelect`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<T>` | - | The selected option; strictly controlled. `None` shows `placeholder` and selects nothing. |
| `onchange` | `EventHandler<T>` | - | Called with the option the caller should select next. Never fires for the placeholder, which cannot be picked. |
| `options` | `Vec<T>` | `T::options()` | Narrows or reorders the list. A runtime set - `String`s, or records fetched from a server - passes them here, since only an enum lists its own. |
| `option_label` | `Callback<T, String>` | `T::label()` | Overrides what the derive named an option. Returns a `String`, not an `OptionLabel`: an `<option>` holds text and nothing else. |
| `placeholder` | `String` | - | Shown while `value` is `None`, as an unpickable first entry. |
| `label` | `Caption` | - | The field's caption, above the control. Names the field through a `for`/`id` pair. |
| `description` | `Caption` | - | Between the label and the control: what to pick. |
| `helper` | `Caption` | - | Under the control: constraints, or what the choice affects. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field. `SelectDefaults`
keeps only what is genuinely this component's: which `size` and `radius` it
starts at.

| Field | Type | Description |
|---|---|---|
| `field.gap` | `&'static str` | Vertical gap between the slots. |
| `field.frame_gap` | `&'static str` | Horizontal gap inside the frame. |
| `field.sizes` | `Sizes<FieldSizeLevel>` | `label_font_size`, `caption_font_size`, `font_size`, `height`, `padding_y`, `padding_x` per size. |
| `select.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `select.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

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
`<select>` itself carries none - the frame styles it.

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
