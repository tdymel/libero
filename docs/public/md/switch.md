# Switch

Crate: `libero`
Import: `use libero::components::Switch;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/switch.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A strictly controlled on/off toggle - a visually hidden checkbox with `role="switch"`, drawn as a track and thumb, wearing the field slots.

A checkbox styled as a track and thumb. A visually hidden `<input>` does the
real work, so it is announced as a switch, and Space and Enter both toggle it.

Since the port onto `use_field` it is a field: the track sits where a
[Checkbox](checkbox.md) puts its box, the label beside it, and the description,
helper text and validation message under both.

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

## Controlled, always

`checked` in, `onchange` out, with no uncontrolled mode. The browser never
toggles the input itself: a click on the label is cancelled, and Space and Enter
are answered on `keydown`, so the switch only moves when its state does - the
browser's own flip never gets to disagree with Rust. `checked` without
`onchange` can never change, and `onchange` without `checked` can never appear
on; the library warns about either alone.

## Migrating from the pre-field Switch

The label used to be `children`. It is now `label`, the same `Caption` every
field takes:

```rust
// before
Switch { checked: on(), onchange: move |next| on.set(next), "Notifications" }
// after
Switch { label: "Notifications", checked: on(), onchange: move |next| on.set(next) }
```

Two other things moved with it. The page is now at `/form/switch`, and
`SwitchSizeLevel` lost its `font_size` - the label scales from `FieldDefaults`
like every other field's, so a switch and a text field in one form read at the
same size.

## Accessibility

Space and Enter toggle it. Without a `label`, pass `aria_label` - but a visible
label is better. With neither, it warns in a debug build.

## Props

### `Switch`

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Track color when checked; a theme color name or a literal CSS color. Unchecked is always `grey.3`. |
| `size` | `Size` | `md` | Controls track and thumb size, and the label beside them. |
| `radius` | `Size` | `xl` | Track corner radius; the thumb is always a circle. |
| `checked` | `bool` | - | Strictly controlled - pair it with `onchange`. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. |
| `label` | `Caption` | - | The caption beside the track. Names the switch through a `for`/`id` pair. |
| `description` | `Caption` | - | Under the label: what turning it on does. |
| `helper` | `Caption` | - | Under the description, in the label's column. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the switch. |
| `aria_label` | `String` | - | Names the switch when it has no `label`. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `name` and `value` among them, since
the props extend `input`'s own.

## Theme defaults

`SwitchDefaults` on the theme; per-size values live in its `sizes` scale. The
label and caption typography comes from `FieldDefaults`.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted (`md`). |
| `radius` | `Size` | Default `radius` when the prop is omitted (`xl`). |
| `sizes` | `Sizes<SwitchSizeLevel>` | `track_width`, `track_height` and `thumb_size` per size - `30x16` with a 12px thumb at `xs`, up the scale from there. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-switch-track-width-<size>` | Track width for that size step. |
| `--lsx-switch-track-height-<size>` | Track height for that size step. |
| `--lsx-switch-thumb-size-<size>` | Thumb diameter for that size step. |
| `--lsx-switch-track-w` / `-track-h` / `--lsx-switch-thumb` | The picked size step, resolved on the control so the track and thumb - which carry no `data-state` - inherit it. |
| `--lsx-switch-radius` | The picked radius step, same mechanism. |
| `--lsx-switch-color` | Track background: the resolved `color` when checked, `grey.3` when not. |
| `--lsx-switch-thumb-color` | Thumb fill: the color's contrast when checked, `white` when not. |
| `--lsx-switch-on` | `0` or `1`, multiplied by the travel distance, so only this var changes between states. |

## Data attributes

`data-state` on the wrapper carries `size-*`, `radius-*`, `inline`, and
`disabled`, `required` and the status token when they apply. The control - the
span holding the input and the track - carries those plus `checked`. The caption
slots are addressed as `data-slot="description" | "helper" | "status"`.
