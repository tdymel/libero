# Textarea

Crate: `libero`
Import: `use libero::components::Textarea;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/textarea.rs>
Index: [index.md](index.md) lists every other page
Description: A multi-line text field, sized by `rows` and resizable by the user.

A multi-line text field with the same slots as [TextField](text_field.md).
`rows` sets the starting height, and the user can drag it taller. With server
rendering, the box stays empty until the app hydrates.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Textarea;

#[component]
fn Demo() -> Element {
    let mut bio = use_signal(String::new);

    rsx! {
        Textarea {
            label: "Bio",
            placeholder: "Start typing",
            rows: 5,
            counter: true,
            maxlength: 200,
            value: bio(),
            oninput: move |next| bio.set(next),
        }
    }
}
```

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `TextareaPart::Label` | `label` | The label above the control. |
| `TextareaPart::Required` | `required` | The required asterisk, in the label. |
| `TextareaPart::Description` | `description` | The caption between the label and the control. |
| `TextareaPart::Frame` | `frame` | The bordered box around the control. |
| `TextareaPart::Control` | `control` | The element the label names. |
| `TextareaPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `TextareaPart::Counter` | `counter` | The `12/200` badge in the frame's corner, with `counter`. |
| `TextareaPart::Helper` | `helper` | The caption under the control. |
| `TextareaPart::Status` | `status` | The validation message. |

## Accessibility

### Libero handles

- The visible counter is hidden from screen readers. Instead, a polite status
  says how many characters are left once a tenth of the limit remains. Its
  words come from the localization's `textarea.characters_left`.

### You must

- Leave `label` unset only when something else names the field.

## Props

### `Textarea`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Padding and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `rows` | `u32` | `3` | Visible lines, which set the starting height. The user can still drag it taller. |
| `value` | `Option<String>` | - | The text in the field. Leave it out and the textarea keeps its own text. |
| `oninput` | `EventHandler<String>` | - | Fires on every keystroke with the text the field should hold next. |
| `validate` | `Validators<String>` | - | Rules over the text, shown once the field loses focus or its form is submitted. |
| `name` | `FieldName<String>` | - | What the field posts as. A path such as `Signup::FIELDS.bio()` also binds the text to the surrounding `Form`'s value when the field has no `oninput`. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `counter` | `bool` | `false` | Shows `12/200` under the control while a `maxlength` attribute is set. It counts as `maxlength` does, so an emoji counts two. |
| `label` | `Caption` | - | The field's caption, above the control. It names the field. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Formatting rules or limits. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. |

`Textarea` also takes the `<textarea>` HTML attributes (`maxlength`, `wrap`,
...) and, like every component, the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

Most of it is `FieldDefaults`, shared by every field. `TextareaDefaults` holds
only the `size` and `radius` it starts at.

| Field | Type | Description |
|---|---|---|
| `textarea.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `textarea.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

The `--lsx-field-*` variables are [TextField](text_field.md)'s, shared unchanged.

## Data attributes

The same as [TextField](text_field.md). The wrapper's and the frame's
`data-state` carry `size-<size>`, `radius-<size>`, `disabled`, `required`,
`warning` and `error`, and the caption slots carry `data-slot`.
