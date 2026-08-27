# TextField

Crate: `libero`
Import: `use libero::components::TextField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/text_field>
Index: [index.md](index.md) - every other component's markdown page
Description: A single-line text field with the five field slots - label, description, control, helper text and validation message.

A single-line text field. It stacks the five slots every field in the library
shares: the label, a description, the control itself, helper text, and a
validation message. Pass `value` to control it and `oninput` to hear about
keystrokes; omit `value` and the `<input>` keeps its own text.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{FieldStatus, TextField};

#[component]
fn Demo() -> Element {
    let mut email = use_signal(String::new);

    rsx! {
        TextField {
            label: "Email",
            description: "The address we send the invoice to.",
            helper: "Work addresses only.",
            status: if email().contains('@') {
                FieldStatus::Valid
            } else {
                FieldStatus::Error("Not a valid address.".to_string())
            },
            required: true,
            placeholder: "ada@example.com",
            value: email(),
            oninput: move |next| email.set(next),
        }
    }
}
```

`size` and `radius` step independently - the captions scale with `size` too -
and `disabled` dims the whole control.

## Controlled and uncontrolled

`value: Some(text)` is controlled: the field renders exactly that text and can
only change through `oninput`, so the caller can rewrite or reject input instead
of racing the DOM for it. Omitting `value` renders no `value` attribute at all,
which leaves the `<input>` to hold its own text - a field that only needs to be
read on submit needs no signal.

`oninput` is the DOM's `input` event: it fires as the user types, not on blur.
A field that should only commit on blur adds `onchange` or `onblur` through the
shared attribute tail, which reach the `<input>` unchanged.

## Captions

`label`, `description` and `helper` are `Caption`s, so each takes either a
string or an `Element`:

```rust
TextField {
    label: "Password",
    helper: rsx! { "At least 8 characters, one " strong { "symbol" } },
}
```

A string caption is given an id and named by the input's `aria-describedby`.
Markup is rendered and styled the same way but names nothing - a caller who
passes markup owns its accessibility.

`status` is separate because a validator produces it. `FieldStatus::Error`
and `FieldStatus::Warning` each carry their message; `&str` and `String` convert
into `Error`, which is the common case, so `status: "Not a valid address."`
works. An error also sets `aria-invalid`; a warning does not, since it would
announce a working field as broken.

## Accessibility

The label is a real `<label for>` paired with the input's `id`, not a wrapping
`<label>` like [Select](select.md) uses - so a control placed inside the field
later cannot have its clicks swallowed by the label. A caller-supplied `id`
takes over the generated one, and every slot follows it.

Whichever of the description, helper and status slots are filled are joined into
the input's `aria-describedby`, in reading order. A caller who passes their own
`aria-describedby` wins outright: theirs is used and the generated list is
dropped, leaving the caption props purely visual.

`required` sets the native attribute and `aria-required`, and adds an asterisk
to the label. The asterisk is `aria-hidden` - `aria-required` already carries it
to assistive technology.

Leave `label` unset only when something else already names the field; a bare
input with no accessible name is a defect. `disabled` sets the native attribute,
so the browser handles focus and interaction semantics.

## What it does not do yet

There are no leading/trailing slots - the chrome still sits directly on the
`<input>`, so nothing can live inside the border. Multi-line input, numbers and
passwords are separate components rather than modes of this one.

## Props

### `TextField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<String>` | - | The text in the field. `None` leaves the `<input>` uncontrolled. |
| `oninput` | `EventHandler<String>` | - | Fires per keystroke with the text the field should hold next. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `label` | `Caption` | - | The field's caption, above the control. Names the field through a `for`/`id` pair. |
| `description` | `Caption` | - | Between the label and the control: what to enter. |
| `helper` | `Caption` | - | Under the control: formatting rules, constraints, counters. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `name`, `readonly`, `maxlength`,
`autocomplete` and `type` among them, since the props extend `input`'s own.

## Theme defaults

Two structs. `FieldDefaults` on the theme carries the slot typography every
field shares; `TextFieldDefaults` carries the control's own scale, which is the
same numbers `SelectDefaults` uses so the two line up in one form.

| Field | Type | Description |
|---|---|---|
| `field.gap` | `&'static str` | Vertical gap between the slots. |
| `field.sizes` | `Sizes<FieldSizeLevel>` | `label_font_size` and `caption_font_size` per size. |
| `text_field.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `text_field.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |
| `text_field.sizes` | `Sizes<TextFieldSizeLevel>` | `font_size`, `height`, `padding_x` per size. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-field-gap` | Vertical gap between the slots. |
| `--lsx-field-label-font-size-<size>` | The label's `font-size` for that size step. |
| `--lsx-field-caption-font-size-<size>` | `font-size` of the description, helper and status text. |
| `--lsx-text-field-font-size-<size>` | `font-size` of the control for that size step. |
| `--lsx-text-field-height-<size>` | `height` for that size step. |
| `--lsx-text-field-padding-x-<size>` | Horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its own.

## Data attributes

State tokens on the wrapper's and the `<input>`'s `data-state`, space separated.

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
