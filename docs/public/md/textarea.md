# Textarea

Crate: `libero`
Import: `use libero::components::Textarea;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/textarea.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A multi-line text field with the five field slots, sized by `rows` and resizable by the user.

A multi-line text field. It stacks the same five slots every field in the
library shares - label, description, the control, helper text and a validation
message - around a `<textarea>` instead of an `<input>`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Textarea;

#[component]
fn Demo() -> Element {
    let mut notes = use_signal(String::new);

    rsx! {
        Textarea {
            label: "Notes",
            placeholder: "Start typing",
            rows: 5,
            value: notes(),
            oninput: move |next| notes.set(next),
        }
    }
}
```

`value` and `oninput` work as they do on a [TextField](text_field.md):
`value: None` leaves the `<textarea>` uncontrolled, and `oninput` fires per
keystroke.

## Height

`rows` sets the starting height, and the browser's own drag handle
(`resize: vertical`) takes it from there. There is no autosize prop: growing the
box to fit the text means measuring `scrollHeight` per keystroke, which is a
platform read this component does not need to work.

The field's height is its padding plus whatever the control needs, so the frame
follows the textarea as it grows. That is what the size scale's `padding_y` is
for - see [TextField](text_field.md) for the shared scale.

## Server-rendered values

A server-rendered `<textarea>` carries its text as a `value` attribute, which
browsers ignore on this element - so the box is empty until the app hydrates and
writes the property. It is the same shape as the gap
[NativeSelect](native_select.md) documents for an ignored `onchange`, and it affects
server-side rendering only.

## Accessibility

Leave `label` unset only when something else already names the field.

## Props

### `Textarea`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls padding and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `rows` | `u32` | `3` | Visible lines, which is what sets the starting height. |
| `value` | `Option<String>` | - | The text in the field. `None` leaves the `<textarea>` uncontrolled. |
| `oninput` | `EventHandler<String>` | - | Fires per keystroke with the text the field should hold next. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `label` | `Caption` | - | The field's caption, above the control. Names the field through a `for`/`id` pair. |
| `description` | `Caption` | - | Between the label and the control: what to enter. |
| `helper` | `Caption` | - | Under the control: formatting rules, constraints, counters. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `name`, `readonly`, `maxlength` and
`wrap` among them, since the props extend `textarea`'s own.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field. `TextareaDefaults`
keeps only which `size` and `radius` it starts at.

| Field | Type | Description |
|---|---|---|
| `textarea.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `textarea.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

The `--lsx-field-*` variables are [TextField](text_field.md)'s, shared unchanged.

## Data attributes

The same as [TextField](text_field.md): state tokens on the wrapper's and the
frame's `data-state` - `size-<size>`, `radius-<size>`, `disabled`, `required`,
`warning`, `error` - and `data-slot` on the caption slots.
