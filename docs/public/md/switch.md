# Switch

Crate: `libero`
Import: `use libero::components::Switch;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/switch.rs>
Index: [index.md](index.md) lists every other page
Description: An on/off toggle drawn as a track and thumb, announced as a switch, with the field slots.

An on/off toggle drawn as a track and thumb. It is a checkbox underneath,
announced as a switch, with the label beside the track and the description,
helper and status under both. Pass `checked` with `onchange` to own the state.
With neither, the switch keeps its own, or the form's when `name` binds it.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Switch;

#[component]
fn Demo() -> Element {
    let mut notifications = use_signal(|| false);

    rsx! {
        Switch {
            label: "Notifications",
            description: "About once a month.",
            checked: notifications(),
            onchange: move |next| notifications.set(next),
        }
    }
}
```

`variant: "card"` draws the whole field as a bordered surface you can click
anywhere. Pair it with a `description`:

```rust,ignore
Switch {
    variant: "card",
    label: "Wi-Fi",
    description: "Joins known networks on its own.",
    checked: wifi(),
    onchange: move |next| wifi.set(next),
}
```

## Props

### `Switch`

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Track color when on. A theme color name or any CSS color. |
| `size` | `Size` | `md` | Size of the track, the thumb and the label. |
| `radius` | `Size` | `xl` | Track corner radius. The thumb stays a circle. |
| `checked` | `bool` | - | Whether it is on. Pair it with `onchange`. Left out, the switch keeps its own state, or the form's when `name` binds it. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. |
| `name` | `FieldName<bool>` | - | What the switch posts as. A path such as `Signup::FIELDS.terms()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<bool>` | - | Rules over `checked`, shown once the switch loses focus or its form is submitted. |
| `label` | `Caption` | - | The caption beside the track, and the switch's name. |
| `description` | `Caption` | - | Under the label. What turning it on does. |
| `helper` | `Caption` | - | Under the description, in the label's column. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Disables and dims the switch. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the switch from the tab order and the post instead. Chromium does not announce read-only on a switch, so say it in the label or description where it matters. |
| `aria_label` | `String` | - | Names the switch when it has no `label`. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws the switch as a bordered surface you can click anywhere. Pair it with a `description`. On the web a link inside the card keeps its own click. Natively the whole card toggles. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes, `value` among them, since the props
extend `input`'s own.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `SwitchPart::Label` | `label` | The label beside the control. |
| `SwitchPart::Required` | `required` | The required asterisk, in the label. |
| `SwitchPart::Description` | `description` | The caption between the label and the control. |
| `SwitchPart::Control` | `control` | Holds the hidden input and the track, beside the label. |
| `SwitchPart::Track` | `track` | The pill the thumb slides along. |
| `SwitchPart::Thumb` | `thumb` | The sliding knob. |
| `SwitchPart::Helper` | `helper` | The caption under the control. |
| `SwitchPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Space` | Toggles the switch. |
| `Enter` | Outside a `Form`: toggles the switch. Inside one: submits the form, as on a native checkbox. |

### Libero handles

- A debug build warns when the switch has neither a visible label nor
  `aria_label`.

### You must

- Without a visible label, set `aria_label`.

### Example

A notifications switch in a settings `Form`, `Switch { label: "Notifications"
}`: Space turns it on or off, Enter submits the form, and the label is its
name.

## Theme defaults

`SwitchDefaults` on the theme. Per-size values live in its `sizes` scale. The
label and caption typography comes from `FieldDefaults`.

| Field | Type | Description |
|---|---|---|
| `variant` | `ChoiceVariant` | Default `variant` when the prop is omitted (`plain`). |
| `size` | `Size` | Default `size` when the prop is omitted (`md`). |
| `radius` | `Size` | Default `radius` when the prop is omitted (`xl`). |
| `sizes` | `Sizes<SwitchSizeLevel>` | `track_width`, `track_height` and `thumb_size` per size, `30x16` with a 12px thumb at `xs`, up the scale from there. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-switch-track-width-<size>` | Track width for that size step. |
| `--lsx-switch-track-height-<size>` | Track height for that size step. |
| `--lsx-switch-thumb-size-<size>` | Thumb diameter for that size step. |
| `--lsx-switch-track-w` / `-track-h` / `--lsx-switch-thumb` | The picked size step, resolved on the control so the track and thumb inherit it. |
| `--lsx-switch-radius` | The picked radius step, resolved the same way. |
| `--lsx-switch-color` | Track background. The resolved `color` when on, `muted.6` when off. |
| `--lsx-switch-thumb-color` | Thumb fill. The color's contrast when on, the surface color when off. |
| `--lsx-switch-on` | `0` off or `1` on. |

## Data attributes

`data-state` on the wrapper carries `size-*`, `radius-*`, `inline`, and
`card`, `disabled`, `required` and the status token when they apply. The control, the
span holding the input and the track, carries those plus `checked`. The caption
slots are addressed as `data-slot="description" | "helper" | "status"`.
