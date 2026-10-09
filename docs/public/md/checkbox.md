# Checkbox

Crate: `libero`
Import: `use libero::components::Checkbox;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/checkbox.rs>
Index: [index.md](index.md) lists every other page
Description: A checkbox with its label beside the box, the field slots under both, and an indeterminate state.

A checkbox with its label beside the box and the description, helper and
status under the label. It has no frame, since the box is the control. Pass
`checked` with `onchange` to own the state. With neither, the box keeps its own,
or the form's when `name` binds it.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Checkbox;

#[component]
fn Demo() -> Element {
    let mut accepted = use_signal(|| false);

    rsx! {
        Checkbox {
            label: "Accept the terms",
            description: "The licence and the privacy policy.",
            checked: accepted(),
            onchange: move |next| accepted.set(next),
        }
    }
}
```

`indeterminate` marks the parent of a partly checked group:

```rust,ignore
Checkbox {
    label: "All channels",
    checked: mail() && sms(),
    indeterminate: (mail() != sms()).then_some(true),
    onchange: move |next| { mail.set(next); sms.set(next); },
}
```

`variant: "card"` draws the whole field as a bordered surface you can click
anywhere. Pair it with a `description`:

```rust,ignore
Checkbox {
    variant: "card",
    label: "Priority support",
    description: "Answers within four hours, around the clock.",
    checked: support(),
    onchange: move |next| support.set(next),
}
```

## Props

### `Checkbox`

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | The box's color when checked. A theme color name or any CSS color; `theme.checkbox.color` when unset. |
| `size` | `Size` | `md` | Size of the box, the label and the captions. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius of the box. Or any CSS, e.g. `radius: "0"`. |
| `checked` | `bool` | - | Whether it is checked. Pair it with `onchange`. Left out, the box keeps its own state, or the form's when `name` binds it. |
| `indeterminate` | `bool` | `false` | Draws a dash and reads as mixed. It wins over `checked`, and toggling from it gives `true`. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. |
| `name` | `FieldName<bool>` | - | What the checkbox posts as. A path such as `Signup::FIELDS.terms()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<bool>` | - | Rules over `checked`, shown once the checkbox loses focus or its form is submitted. |
| `label` | `Caption` | - | The caption beside the box, and the checkbox's name. |
| `description` | `Caption` | - | Under the label. What checking it means. |
| `helper` | `Caption` | - | Under the description, in the label's column. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Disables and dims the checkbox. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the checkbox from the tab order and the post instead. Chromium does not announce read-only on a checkbox, so say it in the label or description where it matters. |
| `aria_label` | `String` | - | Names the checkbox when it has no `label`. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws the checkbox as a bordered surface you can click anywhere. Pair it with a `description`. On the web a link inside the card keeps its own click. Natively the whole card toggles. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes, `value` among them, since the props
extend `input`'s own.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `CheckboxPart::Label` | `label` | The label beside the control. |
| `CheckboxPart::Required` | `required` | The required asterisk, in the label. |
| `CheckboxPart::Description` | `description` | The caption between the label and the control. |
| `CheckboxPart::Control` | `control` | Holds the hidden input and the box, beside the label. |
| `CheckboxPart::Box` | `box` | The drawn square and its mark. |
| `CheckboxPart::Helper` | `helper` | The caption under the control. |
| `CheckboxPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Space` | Toggles the checkbox. |

### You must

- Without a visible label, set `aria_label`.

### Example

A terms checkbox, `Checkbox { label: "Accept the terms" }`: Tab lands on the
box, the label is its name, and Space ticks or clears it.

## Theme defaults

`CheckboxDefaults` holds `variant` (`plain`), `size`, `radius`, and one square
per size step (`14px` to `24px`), published as `--lsx-checkbox-box-size-*`. A
card's padding is `FieldDefaults::card_paddings`, `8px` to `18px`, published as
`--lsx-field-card-padding-*`. The label and caption typography comes from
`FieldDefaults`, so a checkbox and a [TextField](text_field.md) in one form read
at the same scale.

## Data attributes

`data-state` on the wrapper carries `size-*`, `radius-*`, `inline`, `card` for
the card variant, and `disabled`, `required` and the status token when they
apply. The control
carries those plus `checked` or `mixed`. The caption slots are addressed as
`data-slot="description" | "helper" | "status"`.
