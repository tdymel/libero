# Switch

Crate: `libero`
Import: `use libero::components::Switch;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/switch>
Index: [index.md](index.md) - every other component's markdown page
Description: A strictly controlled on/off toggle - a visually hidden checkbox with `role="switch"`, drawn as a track and thumb.

A checkbox styled as a track and thumb. A visually hidden `<input>` does the real
work, so it is announced as a switch and Space toggles it. The root is a `<span>`
holding that input plus a `<label>`, and the track lives *inside* the label - as
a sibling it would be dead to the mouse.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Switch;

#[component]
fn Demo() -> Element {
    let mut notifications = use_signal(|| false);

    rsx! {
        Switch {
            color: "primary",
            size: "md",
            radius: "xl",
            checked: notifications(),
            onchange: move |next| notifications.set(next),
            "Notifications"
        }
    }
}
```

## Controlled

Strictly controlled: `checked` drives the look, `onchange` reports the value it
should take next. The click is cancelled (`prevent_default`), so the switch only
moves when its state does - the browser's own flip never gets to disagree with
Rust. `checked` without `onchange` can never change, and `onchange` without
`checked` can never appear on; the library warns about either alone.

```rust
use dioxus::prelude::*;
use libero::components::{Flex, Switch, Text};

#[component]
fn Demo() -> Element {
    let mut notifications = use_signal(|| true);

    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            Switch {
                checked: notifications(),
                onchange: move |next| notifications.set(next),
                "Notifications"
            }
        }
        Text { "Notifications: {notifications()}" }
    }
}
```

## Without a label

Give it an `aria_label` - `attributes` land on the root, not on the input that
carries the role.

```rust
use dioxus::prelude::*;
use libero::components::Switch;

#[component]
fn Demo() -> Element {
    let mut airplane = use_signal(|| true);

    rsx! {
        Switch {
            aria_label: "Airplane mode",
            checked: airplane(),
            onchange: move |next| airplane.set(next),
        }
    }
}
```

## Accessibility

The real control is an `<input type="checkbox" role="switch">`, so state is
announced and Space toggles it; the `<label>` is wired to it by `for`/`id`, which
is why clicking the track or the text works. A switch with no `children` has no
accessible name unless you pass `aria_label` - an `aria-label` in `attributes`
lands on the root `<span>` and does nothing.

`disabled` sets the input's `disabled` plus `aria-disabled` on the root, dims it,
and removes pointer events. The focus ring is drawn on the *track* rather than the
whole row: the root sets `--lsx-switch-ring` under
`:has(> input:focus-visible)` and the track reads it.

Keep `children` to text and `Icon` - a `<label>` hijacks clicks on any nested
control.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | Track color when checked; a theme color name or a literal CSS color. Unchecked is always `grey.3`. |
| `size` | `Size` | `md` | Controls track, thumb, and label size. |
| `radius` | `Size` | `xl` | Track corner radius; the thumb is always a circle. |
| `checked` | `bool` | - | Strictly controlled - pair it with `onchange`. |
| `disabled` | `bool` | `false` | Disables interaction and dims the switch. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. |
| `aria_label` | `String` | - | Names the switch when it has no `children`; `attributes` cannot, since they land on the root, not the input. |
| `children` | `Element` | - | Text and `Icon` only - a `<label>` hijacks clicks on nested controls. |

Like every component, `Switch` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`SwitchDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted (`md`). |
| `radius` | `Size` | Default `radius` when the prop is omitted (`xl`). |
| `sizes` | `Sizes<SwitchSizeLevel>` | `track_width`, `track_height`, `thumb_size`, `font_size` per size - `30x16` with a 12px thumb at `xs`, up the scale from there. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-switch-track-width-<size>` | Track width for that size step. |
| `--lsx-switch-track-height-<size>` | Track height for that size step. |
| `--lsx-switch-thumb-size-<size>` | Thumb diameter for that size step. |
| `--lsx-switch-font-size-<size>` | Label font size for that size step. |
| `--lsx-switch-track-w` / `-track-h` / `--lsx-switch-thumb` | The picked size step, resolved on the root so the track and thumb - which carry no `data-state` - inherit it. |
| `--lsx-switch-radius` | The picked radius step, same mechanism. |
| `--lsx-switch-color` | Track background: the resolved `color` when checked, `grey.3` when not. |
| `--lsx-switch-thumb-color` | Thumb fill: the color's contrast when checked, `white` when not. |
| `--lsx-switch-on` | `0` or `1`, multiplied by the travel distance, so only this var changes between states. |
| `--lsx-switch-ring` | Set on the root under `:has(> input:focus-visible)` and read by the track, so the ring hugs the track. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `checked` | `checked` is set. |
| `disabled` | `disabled` is set. |
