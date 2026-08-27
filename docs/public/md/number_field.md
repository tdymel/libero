# NumberField

Crate: `libero`
Import: `use libero::components::NumberField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/number_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A numeric field over the caller's own number type, with steppers in its trailing slot.

A numeric field with the five slots every field shares. It is generic over the
value's type: every primitive number implements `NumberValue`, so `T` is
inferred from `value` and there is nothing to write.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::NumberField;

#[component]
fn Demo() -> Element {
    let mut quantity = use_signal(|| Some(1i32));

    rsx! {
        NumberField {
            label: "Quantity",
            helper: "Up to 99 per order.",
            min: 1,
            max: 99,
            value: quantity(),
            onchange: move |next| quantity.set(Some(next)),
        }
    }
}
```

`value: None` is the empty field - a state the type holds rather than an empty
string every caller special-cases. With `None` there is nothing for `T` to be
inferred from, so that call site needs `None::<i32>` and an annotated handler:

```rust
let mut quantity = use_signal(|| None::<i32>);

NumberField {
    label: "Quantity",
    placeholder: "How many?",
    value: quantity(),
    onchange: move |next: i32| quantity.set(Some(next)),
}
```

## The edit buffer

The control is a text input with `inputmode="decimal"`, not `type="number"`. A
number input reports an empty string for anything the browser cannot parse, so
`-` and `1.` vanish as they are typed. This field keeps the raw text in a buffer
instead and publishes only what `T::parse` accepted, so `onchange` never sees a
half-typed number.

The buffer is what the control shows for as long as it still parses to the value
the caller holds. The moment the caller's value says something else, the caller
wins - which is what makes the field controlled.

## Your own number type

`NumberValue` is implemented for `f32`, `f64` and every integer width. A custom
type implements `default_step` and inherits the rest from its `FromStr` and
`Display`, overriding `format` only when the display differs from the parse:

```rust
use libero::components::NumberValue;

/// Money is not an f64: whole cents, shown with a decimal point.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Cents(i64);

// Add, Sub, FromStr and Display as usual.

impl NumberValue for Cents {
    fn default_step() -> Self {
        Cents(50)
    }
}
```

`NumberField { value: price(), onchange: .. }` then works with no further
plumbing, and `onchange` hands back a `Cents`.

The trait's other methods have default bodies: `parse` (`FromStr`), `format`
(`Display`), `zero` (parsing `"0"`, the stepper's starting point for an empty
field), `step_up`/`step_down` (`Add`/`Sub`) and `clamp_to` (`PartialOrd`).
Override one when the type needs it - a wrapping angle, a logarithmic step.

## Steppers and keys

Two buttons in the trailing slot raise and lower the value by `step`, defaulting
to `T::default_step()`. Arrow Up and Arrow Down do the same from the keyboard,
with the default prevented so the caret does not jump. Both paths clamp to
`min`/`max`, and an empty field steps from `T::zero()`.

`increment_label` and `decrement_label` name the buttons; the glyphs are
`aria-hidden`.

## Accessibility

`min` and `max` mean nothing on a text input, so the control carries
`role="spinbutton"` with `aria-valuenow`, `aria-valuemin` and `aria-valuemax` -
which is what tells assistive technology the range, and what makes the arrow
keys expected behaviour rather than a surprise.

The rest is [TextField](text_field.md)'s: a `<label for>`/`id` pair, the filled
caption slots joined into `aria-describedby`, `aria-invalid` on an error status,
and the focus ring drawn by the frame around the control.

## Props

### `NumberField<T: NumberValue>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<T>` | - | The number in the field; strictly controlled. `None` is the empty field. |
| `onchange` | `EventHandler<T>` | - | Called with the number the caller should hold next. |
| `min` | `Option<T>` | - | Floor, enforced on typing and on the steppers alike. |
| `max` | `Option<T>` | - | Ceiling, same. |
| `step` | `Option<T>` | `T::default_step()` | What one press of a stepper moves by. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `increment_label` | `String` | `Increase` | Announced on the stepper that raises the value. |
| `decrement_label` | `String` | `Decrease` | Announced on the stepper that lowers it. |
| `label` | `Caption` | - | The field's caption, above the control. |
| `description` | `Caption` | - | Between the label and the control: what to enter. |
| `helper` | `Caption` | - | Under the control: units, ranges, what the number means. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes, since the props extend `input`'s own.

### `NumberValue`

| Method | Default | Description |
|---|---|---|
| `default_step()` | required | The step a press moves by when `step` is unset. |
| `parse(&str)` | `FromStr` | `None` while the buffer is not yet a number. |
| `format(&self)` | `Display` | What the control shows for a value from outside. |
| `zero()` | `parse("0")` | Where a stepper starts from in an empty field. |
| `step_up`/`step_down` | `Add`/`Sub` | One step in each direction. |
| `clamp_to(min, max)` | `PartialOrd` | The value pulled into the field's range. |

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field. `NumberFieldDefaults`
keeps only which `size` and `radius` it starts at.

| Field | Type | Description |
|---|---|---|
| `number_field.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `number_field.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

The `--lsx-field-*` variables are [TextField](text_field.md)'s, shared unchanged.

## Data attributes

The same as [TextField](text_field.md): state tokens on the wrapper's and the
frame's `data-state`, and `data-slot="trailing"` on the steppers' slot.
