# NativeSelect

Crate: `libero`
Import: `use libero::components::{Options, NativeSelect};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/native_select.rs>
Index: [index.md](index.md) lists every other page
Description: A styled native `<select>` over an enum, with the field slots.

A styled native select over an enum, with a label, captions and a status like
every field. The options are the enum's variants, so `onchange` hands back the
value itself. `value` is an `Option`, and `None` is a field nobody has filled in
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

While `value` is `None`, the `placeholder` shows as the selected entry. It
cannot be picked again once a real value is:

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

A runtime set, such as `String`s or records from a server, goes in `options`.
Any type can be an option by implementing `Options`. `label` is the only
required method. `value` is what each `<option>` posts and defaults to `label`,
so no two options may share one. For an enum the derive posts the variant's
name.

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

`onchange` hands back the `Order`. The select finds the selected option with
`PartialEq`, so make it compare identity, not content. An `Order` that derives
`PartialEq` over every field loses its selection when one of them changes.

Labels that need data the value lacks go through `option_label`, which runs
during render:

```rust,ignore
NativeSelect {
    value: selected(),
    options: order_ids(),
    option_label: move |id: OrderId| orders.read().title_of(id),
    onchange: move |id| selected.set(Some(id)),
}
```

`options` also takes an `OptionList`, as on [Select](select.md). A disabled
`OptionItem` renders as `<option disabled>`, and a named group as an
`<optgroup>`:

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

## Props

### `NativeSelect`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `Size` | `sm` | Corner radius. |
| `value` | `Option<T>` | - | The selected option. Pair it with `onchange`. `None` shows `placeholder` and selects nothing. |
| `onchange` | `EventHandler<T>` | - | Called with the option to select next. Never for the placeholder, which cannot be picked. |
| `name` | `FieldName<Option<T>>` | - | What the select posts as. A path such as `Order::FIELDS.size()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Option<T>>` | - | Rules over the selection, shown once the select loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList` adds disabled options and named groups. A pending source draws no options. |
| `option_label` | `Callback<T, String>` | `T::label()` | Renames an option. Returns a `String`, since an `<option>` holds only text. |
| `placeholder` | `String` | - | Shown while `value` is `None`, as a first entry that cannot be picked. |
| `label` | `Caption` | - | The caption above the control, and the select's name. A string or an `Element`. |
| `description` | `Caption` | - | Between the label and the control. What to pick. |
| `helper` | `Caption` | - | Under the control. Constraints, or what the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. An untouched select is not announced invalid. Inside a `Form`, an unpicked one fails the submit. |
| `disabled` | `bool` | `false` | Disables and dims the field. A native `<select>` has no read-only state, so there is no `readonly`. Use `Select` for that. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FieldPart::Label` | `label` | The label above the control. |
| `FieldPart::Required` | `required` | The required asterisk, in the label. |
| `FieldPart::Description` | `description` | The caption between the label and the control. |
| `FieldPart::Frame` | `frame` | The bordered box around the control. |
| `FieldPart::Control` | `control` | The element the label names. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

## Accessibility

### Libero handles

- Your own `aria-describedby` ids come first, before the captions.

### You must

- Without a visible `label`, set `aria_label`. A select with no name is a
  defect.

### Example

A font size select with no visible label, `NativeSelect { aria_label: "Size"
}`: a screen reader names it "Size", and the browser's own list handles the
arrows and type-ahead.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field.
`NativeSelectDefaults` holds only the starting `size` and `radius`.

| Field | Type | Description |
|---|---|---|
| `field.gap` | `&'static str` | Vertical gap between the slots. |
| `field.frame_gap` | `&'static str` | Horizontal gap inside the frame. |
| `field.sizes` | `Sizes<FieldSizeLevel>` | `label_font_size`, `caption_font_size`, `font_size`, `height`, `padding_y`, `padding_x` per size. |
| `native_select.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `native_select.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

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
`<select>` itself also carries `data-controlled`, and `data-placeholder` while no
option matches `value` (`None`, or an option not in the list yet), which dims it.

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
