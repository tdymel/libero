# TextField

Crate: `libero`
Import: `use libero::components::TextField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/text_field.rs>
Index: [index.md](index.md) lists every other page
Description: A single-line text field with a label, a description, helper text and a validation message.

A single-line text field. Like every field, it stacks a label, a description,
the control, helper text and a validation message. Pass `value` and `oninput`
to control it, or leave `value` out and the input keeps its own text. `leading`
and `trailing` put content inside the border, beside the control.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{FieldStatus, TextField};

#[component]
fn Demo() -> Element {
    let mut handle = use_signal(String::new);

    rsx! {
        TextField {
            label: "Username",
            description: "How other people see you.",
            helper: "Letters, numbers and underscores.",
            status: if handle().contains(' ') {
                FieldStatus::Error("No spaces.".to_string())
            } else {
                FieldStatus::Valid
            },
            required: true,
            placeholder: "ada",
            leading: rsx! { "@" },
            trailing: rsx! { "{handle().chars().count()}/20" },
            maxlength: 20,
            describe_trailing: true,
            value: handle(),
            oninput: move |next| handle.set(next),
        }
    }
}
```

## Props

### `TextField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<String>` | - | The text in the field. Leave it out and the input keeps its own text. |
| `oninput` | `EventHandler<String>` | - | Fires on every keystroke with the text the field should hold next. |
| `validate` | `Validators<String>` | - | Rules over the text, shown once the field loses focus or its form is submitted. |
| `name` | `FieldName<String>` | - | What the field posts as. A path such as `Signup::FIELDS.email()` also binds the text to the surrounding `Form`'s value when the field has no `oninput`. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `leading` | `Element` | - | Inside the frame, before the control, such as a search icon or a currency sign. |
| `trailing` | `Element` | - | Inside the frame, after the control, such as a clear button or a unit. |
| `describe_leading` | `bool` | `false` | Set it when `leading` is text that belongs to the value, such as `@`, so a screen reader reads it with the input. Not for an icon or a button. |
| `describe_trailing` | `bool` | `false` | The same for `trailing`, such as `kg` or a `12/20` counter. |
| `label` | `Caption` | - | The field's caption, above the control. It names the field. Takes a string or an `Element`. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Formatting rules, limits or a counter. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. |

`TextField` also takes the `<input>` HTML attributes (`maxlength`,
`autocomplete`, `type`, ...) and, like every component, the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

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
| `FieldPart::Leading` | `leading` | The slot before the control: an icon, a prefix. |
| `FieldPart::Control` | `control` | The element the label names. |
| `FieldPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

## Accessibility

### Libero handles

- A string `description` or `helper` is read with the input.
- An error status marks the input invalid, a warning does not.

### You must

- Leave `label` unset only when something else names the field, such as an
  `aria_label`.
- Markup in `description` or `helper` is shown but not read, so its
  accessibility is yours.
- A `leading` or `trailing` slot is not read with the input. When it is text
  that belongs to the value, such as a unit or a counter, set
  `describe_leading` or `describe_trailing`.

## Theme defaults

Most of it is `FieldDefaults`, shared by every field. `TextFieldDefaults` holds
only the `size` and `radius` it starts at.

| Field | Type | Description |
|---|---|---|
| `field.gap` | `&'static str` | Vertical gap between the slots. |
| `field.frame_gap` | `&'static str` | Horizontal gap between leading, control and trailing. |
| `field.sizes` | `Sizes<FieldSizeLevel>` | `label_font_size`, `caption_font_size`, `font_size`, `height`, `padding_y`, `padding_x` per size. |
| `text_field.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `text_field.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-field-gap` | Vertical gap between the slots. |
| `--lsx-field-label-font-size-<size>` | The label's `font-size` for that size step. |
| `--lsx-field-caption-font-size-<size>` | `font-size` of the description, helper and status text. |
| `--lsx-field-frame-gap` | Horizontal gap inside the frame. |
| `--lsx-field-font-size-<size>` | `font-size` of the control for that size step. |
| `--lsx-field-height-<size>` | The frame's `min-height` for that size step. |
| `--lsx-field-padding-y-<size>` | The frame's vertical padding, which sets the height once the control wraps. |
| `--lsx-field-padding-x-<size>` | The frame's horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale.

## Data attributes

State tokens on the wrapper's and the frame's `data-state`, space separated.
The `<input>` itself carries none.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `disabled` | `disabled` is set. |
| `required` | `required` is set. |
| `warning` | `status` is a `Warning`. |
| `error` | `status` is an `Error`. |

The caption slots carry `data-slot="description"`, `"helper"`, `"status"` and
`"required"`. `leading` and `trailing` carry theirs inside the frame.
