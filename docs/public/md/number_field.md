# NumberField

Crate: `libero`
Import: `use libero::components::NumberField;`
Feature: `number-field` (off by default, turns on `action-icon`, `button`, `paper`, `use-drag`, `use-media-query`, `use-popover`, `use-swipe`, `use-timers`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/number_field.rs>
Index: [index.md](index.md) lists every other page
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
            value: quantity(),
            onchange: move |next| quantity.set(next),
            label: "Quantity",
            steppers: true,
            min: 1,
            max: 99,
        }
    }
}
```

## Props

### `NumberField<T: NumberValue>`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. Or any CSS, e.g. `radius: "0"`. |
| `value` | `Option<T>` | - | The number in the field, strictly controlled. `None` is the empty field. A signal that starts at `None` needs its type, such as `None::<i32>`. |
| `onchange` | `EventHandler<Option<T>>` | - | Called with the number the caller should hold next, `None` once the field is emptied. Half-typed text such as `-` never reaches it; a float `1.` already parses, so it arrives as `1`. Leaving the field clamps a number out of range and reverts text that never parsed. |
| `validate` | `Validators<Option<T>>` | - | Rules over the number, shown once the field loses focus or its form is submitted. |
| `min` | `Option<T>` | - | Floor. Steps clamp to it. Typed text below it clamps once the field is left or Enter is pressed. |
| `max` | `Option<T>` | - | Ceiling, the same way. |
| `step` | `Option<T>` | `T::default_step()` | What one step moves by, `1` for an integer, `1.0` for a float. |
| `name` | `FieldName<Option<T>>` | - | What the field posts as. A path such as `Signup::FIELDS.age()` also binds the number to the surrounding `Form`'s value when the field has no `onchange`. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `steppers` | `bool` | `false` | Shows minus and plus buttons in the trailing slot. The arrow keys step the value either way. |
| `group_digits` | `bool` | `false` | Writes the thousands grouped in the provider's `Formats`, `1,234,567.5` or `1.234.567,5`, while the field is not focused; typing sees the bare number. Built-in number types only. Grouped text typed or pasted parses either way. A WebView (desktop, Android) cannot read or set the caret, so focusing the field may send it to the end of the text. |
| `decimals` | `Decimals` | - | How many digits the fraction keeps. `Decimals::AtMost(2)` cuts a third typed digit and rounds a step or a caller's value, so `0.1 + 0.2` arrives as `0.3`; `Decimals::Exactly(2)` also pads the shown number, `12.50`, while nothing is being typed. `f32` and `f64` only. |
| `leading` | `Element` | - | Inside the frame, before the control: a currency sign such as `$`. Outside the typed text, so typing and parsing stay plain. |
| `trailing` | `Element` | - | Inside the frame, after the control and before the steppers: a unit, `%` or `€`. Outside the typed text. |
| `describe_leading` | `bool` | `false` | `leading` is text that describes the value, so the field's `aria-describedby` points at it. |
| `describe_trailing` | `bool` | `false` | `trailing` is text that describes the value, e.g. a unit. |
| `increment_label` | `String` | `number_field.increase` | Names the plus button, such as "Add a guest". Unset, the localization's `number_field.increase`, "Increase" in English. |
| `decrement_label` | `String` | `number_field.decrease` | Names the minus button. Unset, the localization's `number_field.decrease`, "Decrease" in English. |
| `touch_input` | `TouchInput` | `Keyboard` | What a tap under a finger opens. `TouchInput::Spin` opens a spin column that steps by `step`, in a sheet below `sm` and a popover from `sm` up, with no soft keyboard; a mouse still types. A drag, the wheel, a tap on a neighbour or an arrow key steps it and calls `onchange` at once; Done, Escape, Back and a tap outside only close it. Without `min` and `max` it spins on to the type's limits, and an empty field starts at `min`, else zero. The keyboard button beside Done switches to typing: it closes the spinner and focuses the field with its keypad. A phone browser raises the keyboard at once; in an Android app the next tap on the field does. |
| `label` | `Caption` | - | The field's caption, above the control. It names the field. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Units, ranges, what the number means. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label; a field without a visible `label` shows no mark. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Disables and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. |

`NumberField` also takes the `<input>` HTML attributes and, like every
component, the shared props `sx`, `class`, `style`, `states`, and any extra
HTML attributes.

### `NumberValue`

| Method | Type | Default | Description |
|---|---|---|---|
| `default_step` | `fn() -> Self` | required | What one step moves by when `step` is unset. |
| `parse` | `fn(&str) -> Option<Self>` | `FromStr` | `None` while the text is not a number yet. `f64`'s `FromStr` also reads `nan` and `inf`, so a type built on it should refuse them. |
| `format` | `fn(&self) -> String` | `Display` | The text the field shows for a value. |
| `zero` | `fn() -> Option<Self>` | `parse("0")` | Where a step starts in an empty field. `None` makes the step do nothing. |
| `step_up` | `fn(self, Self) -> Self` | `Add` | One step up. Override it for a wrapping angle or a logarithmic step. |
| `step_down` | `fn(self, Self) -> Self` | `Sub` | One step down. |
| `clamp_between` | `fn(self, Option<Self>, Option<Self>) -> Self` | `PartialOrd` | The value pulled into the field's range. |

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
| `FieldPart::Leading` | `leading` | The slot before the control: a currency sign. |
| `FieldPart::Control` | `control` | The element the label names. |
| `FieldPart::Trailing` | `trailing` | The slot after the control: a unit and the steppers. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Up` or `Down` | Step the value, with or without `steppers`. |
| `PageUp` or `PageDown` | Step the value ten steps. |
| `Home` or `End` | In the touch spinner, go to `min` or `max` when set. |
| `Enter` or `Escape` | Close the touch spinner and return focus to the field. |

### Libero handles

- The touch spinner is a `spinbutton` named by the field's label and described
  by a `describe_trailing` unit. Opening it focuses the column; Done, Escape,
  Android's Back and a tap outside close it and return focus to the field.
- The spinner's neighbours and Done are single-pointer alternatives to its drag,
  and the neighbours are 44px tall under a finger.
- The stepper buttons are not tab stops, since the arrow keys do the same from
  the field.
- The phone keypad follows the type and `min`: digits for a whole number from 0,
  digits and a point for a fraction from 0, the full keyboard where a negative
  is allowed, since iOS keypads have no minus. An `inputmode` attribute
  overrides it.

### You must

- Leave `label` unset only when something else names the field.
- With `steppers` on several fields of one form, name the buttons by their
  field with `increment_label` and `decrement_label`, such as "Add a guest".
  Unset, every field's are "Increase" and "Decrease".

### Example

Two guest counts with `steppers` in one form: ArrowUp adds one guest, PageUp
ten, and `increment_label: "Add an adult"` and `"Add a child"` tell the two
Increase buttons apart.

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
