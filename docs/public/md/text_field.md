# TextField

Crate: `libero`
Import: `use libero::components::TextField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/text_field.rs>
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

## Inside the frame

The border, background, radius and padding belong to a frame around the
`<input>`, not to the input itself, so `leading` and `trailing` can put content
inside it:

```rust
TextField {
    label: "Username",
    placeholder: "ada",
    leading: rsx! { "@" },
    trailing: rsx! { "{handle().len()}/20" },
}
```

Both take any `Element` - text, an `Icon`, an `ActionIcon` for something
clickable. They sit either side of the control, vertically centred, and do not
shrink, so the control takes the space that is left.

Keep the two out of the placeholder's way: a `trailing` of `".com"` above a
placeholder of `ada@example.com` says the same thing twice.

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

Leave `label` unset only when something else already names the field; a bare
input with no accessible name is a defect. A caller-supplied
`aria-describedby` replaces the one built from the caption slots, leaving them
purely visual.

## What it does not do yet

Multi-line input, numbers and passwords are separate components rather than
modes of this one.

## Props

### `TextField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<String>` | - | The text in the field. `None` leaves the `<input>` uncontrolled. |
| `oninput` | `EventHandler<String>` | - | Fires per keystroke with the text the field should hold next. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `leading` | `Element` | - | Inside the frame, before the control. |
| `trailing` | `Element` | - | Inside the frame, after the control. |
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

Almost everything is `FieldDefaults`, shared by every field. `TextFieldDefaults`
keeps only what is genuinely this component's: which `size` and `radius` it
starts at.

| Field | Type | Description |
|---|---|---|
| `field.gap` | `&'static str` | Vertical gap between the slots. |
| `field.frame_gap` | `&'static str` | Horizontal gap between leading, control and trailing. |
| `field.sizes` | `Sizes<FieldSizeLevel>` | `label_font_size`, `caption_font_size`, `font_size`, `height`, `padding_y`, `padding_x` per size. |
| `text_field.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `text_field.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-field-gap` | Vertical gap between the slots. |
| `--lsx-field-label-font-size-<size>` | The label's `font-size` for that size step. |
| `--lsx-field-caption-font-size-<size>` | `font-size` of the description, helper and status text. |
| `--lsx-field-frame-gap` | Horizontal gap inside the frame. |
| `--lsx-field-font-size-<size>` | `font-size` of the control for that size step. |
| `--lsx-field-height-<size>` | The frame's `min-height` for that size step - the floor a single-line field sits at. |
| `--lsx-field-padding-y-<size>` | The frame's vertical padding, which sets the height once the control wraps. |
| `--lsx-field-padding-x-<size>` | The frame's horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its own.

## Data attributes

State tokens on the wrapper's and the frame's `data-state`, space separated. The
`<input>` itself carries none - the frame styles it.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `disabled` | `disabled` is set. |
| `required` | `required` is set. |
| `warning` | `status` is a `Warning`. |
| `error` | `status` is an `Error`. |

The caption slots carry `data-slot="description"`, `"helper"`, `"status"` and
`"required"`, which is how the wrapper styles them; `leading` and `trailing`
carry theirs inside the frame.
