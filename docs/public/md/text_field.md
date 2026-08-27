# TextField

Crate: `libero`
Import: `use libero::components::TextField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/text_field>
Index: [index.md](index.md) - every other component's markdown page
Description: A single-line text field with its own caption, strictly controlled by `value` plus `onchange`.

A single-line text field. Strictly controlled: `value` is what it shows, and
`onchange` fires per keystroke with the text it should hold next - so the caller
can rewrite or reject input instead of racing the DOM for it. There is no
uncontrolled mode.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::TextField;

#[component]
fn Demo() -> Element {
    let mut name = use_signal(String::new);

    rsx! {
        TextField {
            label: "Name",
            placeholder: "Ada Lovelace",
            value: name(),
            onchange: move |next| name.set(next),
        }
    }
}
```

`size` and `radius` step independently - the caption scales with `size` too -
and `disabled` dims the whole control. `onchange` is the DOM's `input` event,
not its `change`: it fires as the user types rather than on blur, and a field
that should only commit on blur adds `onblur` through the shared attribute tail.

## What it does not do yet

There are no leading/trailing slots, no `error`/`description` text, and no
variants - the chrome sits directly on the `<input>`. Multi-line input, numbers
and passwords are separate components rather than modes of this one.

## Accessibility

The caption is a real `<label for>` paired with the input's `id`, not a wrapping
`<label>` like [Select](select.md) uses - so a control placed inside the field
later cannot have its clicks swallowed by the label. A caller-supplied `id`
takes over the generated one, and both the label and the input follow it.

Leave `label` unset only when something else already names the field; a bare
input with no accessible name is a defect. `disabled` sets the native attribute,
so the browser handles focus and interaction semantics.

## Props

### `TextField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `Option<String>` | - | The text in the field; strictly controlled. `None` is the empty field. |
| `onchange` | `EventHandler<String>` | - | Called per keystroke with the text the field should hold next. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `disabled` | `bool` | `false` | Disables interaction and dims the field. |
| `label` | `String` | - | The field's own caption, above the control. Names the field through a `for`/`id` pair. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `name`, `readonly`, `maxlength`,
`autocomplete` and `type` among them, since the props extend `input`'s own.

## Theme defaults

`TextFieldDefaults` on the theme; per-size values live in its `sizes` scale, and
they are the same numbers `SelectDefaults` uses so the two line up in one form.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |
| `sizes` | `Sizes<TextFieldSizeLevel>` | `font_size`, `height`, `padding_x`, `label_font_size` per size. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-text-field-font-size-<size>` | `font-size` for that size step. |
| `--lsx-text-field-height-<size>` | `height` for that size step. |
| `--lsx-text-field-padding-x-<size>` | Horizontal padding for that size step. |
| `--lsx-text-field-label-font-size-<size>` | The caption's `font-size` for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its own.

## Data attributes

State tokens on the `<input>`'s `data-state`, and on the caption's, space
separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `disabled` | `disabled` is set. |
