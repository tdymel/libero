# ToggleButtonGroup

Crate: `libero`
Import: `use libero::components::{ToggleButton, ToggleButtonGroup};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/toggle_button_group>
Index: [index.md](index.md) - every other component's markdown page
Description: A row of connected buttons sharing one selection, for choosing between a handful of options.

A row of connected buttons sharing one selection. Each member is a real
`<button>` carrying `aria-pressed`, so tab order and Space/Enter are the
browser's own. Strictly controlled: `value` drives the look, `onchange` reports
the selection the group should take next.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{ToggleButton, ToggleButtonGroup};

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| vec!["left".to_string()]);

    rsx! {
        ToggleButtonGroup {
            value: alignment(),
            onchange: move |next| alignment.set(next),
            ToggleButton { value: "left", "Left" }
            ToggleButton { value: "center", "Center" }
            ToggleButton { value: "right", "Right" }
        }
    }
}
```

`exclusive` is on by default, so picking one button clears the rest; turn it off
and `value` holds every pressed button. Setting `gap` stops the buttons sharing
borders - each keeps its own border and its own radius.

## Per-button disabled

`disabled` on the group covers every button; a single `ToggleButton` can set its
own instead.

```rust
use dioxus::prelude::*;
use libero::components::{ToggleButton, ToggleButtonGroup};

#[component]
fn Demo() -> Element {
    let mut selected = use_signal(|| vec!["a".to_string()]);

    rsx! {
        ToggleButtonGroup {
            value: selected(),
            onchange: move |next| selected.set(next),
            ToggleButton { value: "a", "One" }
            ToggleButton { value: "b", disabled: true, "Two" }
        }
    }
}
```

## Accessibility

The group is a `role="group"` of ordinary buttons, each carrying
`aria-pressed`. Tab reaches every button and Space or Enter toggles it - there
is no arrow-key navigation to learn, and none is added.

For a single-select group, APG would prefer a `radiogroup`. Following MUI, this
component stays with pressed buttons: the roving tabindex a radiogroup requires
costs more than it returns here, and a toggle button reads correctly to a screen
reader either way. Name the group with an `aria_label` where its purpose is not
obvious from the buttons themselves.

```rust
use dioxus::prelude::*;
use libero::components::{ToggleButton, ToggleButtonGroup};

#[component]
fn Demo() -> Element {
    let mut alignment = use_signal(|| vec!["left".to_string()]);

    rsx! {
        ToggleButtonGroup {
            aria_label: "Text alignment",
            value: alignment(),
            onchange: move |next| alignment.set(next),
            ToggleButton { value: "left", "Left" }
            ToggleButton { value: "center", "Center" }
            ToggleButton { value: "right", "Right" }
        }
    }
}
```

## Props

### ToggleButtonGroup

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Vec<String>` | - | Strictly controlled - pair it with `onchange`. `exclusive` holds it to at most one entry. |
| `onchange` | `EventHandler<Vec<String>>` | - | Called with the selection the group should take next. |
| `exclusive` | `bool` | `true` | Single-select: picking one clears the rest. |
| `orientation` | `Orientation` | `horizontal` | Row or column layout. |
| `variant` | `ButtonVariant` | `outlined` | The unselected look, passed to every button. |
| `color` | `ThemeAwareValue` | `primary` | Accent color; a theme color name or a literal CSS color. |
| `size` | `Size` | `md` | Passed to every button. |
| `radius` | `Size` | `md` | Corner radius of the group's outer corners; inner ones are square. |
| `gap` | `Size` | - | Space between the buttons. Set it and they stop sharing borders - each keeps its own, and its own radius. |
| `full_width` | `bool` | `false` | Buttons share the width evenly instead of sizing to their label. |
| `disabled` | `bool` | `false` | Disables every button in the group. |
| `children` | `Element` | required | `ToggleButton`s. |

### ToggleButton

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `String` | required | Its identity in the group's selection. |
| `disabled` | `bool` | follows the group | Overrides the group's `disabled` for this button alone. |
| `children` | `Element` | required | The button's label. |

Like every component, both also take the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

None of its own. Every visual prop is forwarded to the buttons, so the group
inherits [Button](button.md)'s `ButtonDefaults`; `gap` resolves against the
theme's spacing scale.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-spacing-<size>` | Read for `gap` when it is set. |

Everything else is the buttons' own - see [Button](button.md).

## Data attributes

State tokens on the group root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `horizontal` / `vertical` | The `orientation` in effect. |
| `full-width` | `full_width` is set. |
| `collapsed` | No `gap` - the buttons share borders and square off their inner corners. |
| `size-<size>` | The `gap` step in effect, when `gap` is set. |

Each `ToggleButton` carries `Button`'s own tokens, including `checked` when it
is part of the selection.
