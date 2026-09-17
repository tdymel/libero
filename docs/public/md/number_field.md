# NumberField

Crate: `libero`
Import: `use libero::components::NumberField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/number_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A numeric field over your own number type, with optional steppers in its trailing slot.

A numeric field over your own number type, with optional steppers. Every
primitive number implements `NumberValue`, so `T` is inferred from `value`. A
type of your own implements `default_step` and gets the rest from `FromStr` and
`Display`. A float field writes and reads the decimal separator of the
provider's `Formats`, so `1,5` under `Formats::GERMAN`.

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
            steppers: true,
            min: 1,
            max: 99,
            value: quantity(),
            onchange: move |next| quantity.set(next),
        }
    }
}
```

A custom type, here whole cents shown with a decimal point:

```rust
use dioxus::prelude::*;
use libero::components::{NumberField, NumberValue};
use std::fmt;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Cents(i64);

impl std::ops::Add for Cents {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Cents(self.0 + other.0)
    }
}

impl std::ops::Sub for Cents {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Cents(self.0 - other.0)
    }
}

impl std::str::FromStr for Cents {
    type Err = std::num::ParseFloatError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Cents((text.parse::<f64>()? * 100.0).round() as i64))
    }
}

impl fmt::Display for Cents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl NumberValue for Cents {
    fn default_step() -> Self {
        Cents(50)
    }
}

#[component]
fn Demo() -> Element {
    let mut price = use_signal(|| None::<Cents>);

    rsx! {
        NumberField {
            label: "Price",
            placeholder: "0.00",
            value: price(),
            onchange: move |next| price.set(next),
        }
    }
}
```

## Accessibility

Arrow Up and Arrow Down step the value, Page Up and Page Down ten steps, with
or without `steppers`. The stepper buttons are not tab stops, since the arrow
keys do the same from the field. Leave `label` unset only when something else
names the field.

## Props

### `NumberField<T: NumberValue>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<T>` | - | The number in the field, strictly controlled. `None` is the empty field. A signal that starts at `None` needs its type, such as `None::<i32>`. |
| `onchange` | `EventHandler<Option<T>>` | - | Called with the number the caller should hold next, `None` once the field is emptied. Half-typed text such as `-` or `1.` never reaches it. Leaving the field clamps a number out of range and reverts text that never parsed. |
| `validate` | `Validators<Option<T>>` | - | Rules over the number, shown once the field loses focus or its form is submitted. |
| `min` | `Option<T>` | - | Floor. Steps clamp to it. Typed text below it clamps once the field is left or Enter is pressed. |
| `max` | `Option<T>` | - | Ceiling, the same way. |
| `step` | `Option<T>` | `T::default_step()` | What one step moves by, `1` for an integer, `1.0` for a float. |
| `name` | `FieldName<Option<T>>` | - | What the field posts as. A path such as `Signup::FIELDS.age()` also binds the number to the surrounding `Form`'s value when the field has no `onchange`. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `steppers` | `bool` | `false` | Shows minus and plus buttons in the trailing slot. The arrow keys step the value either way. |
| `increment_label` | `String` | `number_field.increase` | Names the plus button, such as "Add a guest". Unset, the localization's `number_field.increase`, "Increase" in English. |
| `decrement_label` | `String` | `number_field.decrease` | Names the minus button. Unset, the localization's `number_field.decrease`, "Decrease" in English. |
| `label` | `Caption` | - | The field's caption, above the control. It names the field. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Units, ranges, what the number means. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. |

`NumberField` also takes the `<input>` HTML attributes and, like every
component, the shared props `sx`, `class`, `style`, `states`, and any extra
HTML attributes.

### `NumberValue`

| Method | Type | Default | Description |
|---|---|---|---|
| `default_step` | `fn() -> Self` | required | What one step moves by when `step` is unset. |
| `parse` | `fn(&str) -> Option<Self>` | `FromStr` | `None` while the text is not a number yet. |
| `format` | `fn(&self) -> String` | `Display` | The text the field shows for a value. |
| `zero` | `fn() -> Option<Self>` | `parse("0")` | Where a step starts in an empty field. `None` makes the step do nothing. |
| `step_up` | `fn(self, Self) -> Self` | `Add` | One step up. Override it for a wrapping angle or a logarithmic step. |
| `step_down` | `fn(self, Self) -> Self` | `Sub` | One step down. |
| `clamp_between` | `fn(self, Option<Self>, Option<Self>) -> Self` | `PartialOrd` | The value pulled into the field's range. |

## Theme defaults

Most of it is `FieldDefaults`, shared by every field. `NumberFieldDefaults`
holds only the `size` and `radius` it starts at.

| Field | Type | Description |
|---|---|---|
| `number_field.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `number_field.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

The `--lsx-field-*` variables are [TextField](text_field.md)'s, shared unchanged.

## Data attributes

The same as [TextField](text_field.md). The steppers' slot carries
`data-slot="trailing"`.
