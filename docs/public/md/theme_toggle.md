# ThemeToggle

Crate: `libero`
Import: `use libero::components::ThemeToggle;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/theme_toggle.rs>
Index: [index.md](index.md) lists every other page
Description: An icon button that flips the colour scheme, optionally through the system's too, with an optional theme picker beside it.

An icon button that switches the app's colour scheme. It starts on the
system's scheme, and each press flips to the other one. Flipping back to the
system's scheme follows the system again, so the app never stays pinned. The
icon shows where the next press goes: a sun for light, a moon for dark.

With `with_system`, following the system is a step of its own. A press goes
from following the system, to the scheme the system is not showing, to the one
it is, and back, so under a light system the order is system, dark, light. The
icon for that step is a half-filled disc.

While it follows the system, a change of the OS setting applies at once. A
picked scheme stays until the next press. For your own control, such as a menu
of all three choices, build on `use_color_scheme()`: it reads the setting and
what it resolves to, and `set`, `toggle` and `cycle` change it.

With `themes` set, a second button beside it opens a menu of theme sets. This
site's header uses `ThemeToggle { themes: ThemeSet::CATALOGUE }`. The
button names come from the [localization](localization.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::ThemeToggle, theme::ThemeSet};

#[component]
fn Demo() -> Element {
    rsx! {
        ThemeToggle { themes: ThemeSet::CATALOGUE }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | `outlined` | Visual style, as on `ActionIcon`. |
| `color` | `ThemeAwareValue` | `muted` | Accent color. A theme color name or any CSS color. |
| `size` | `ThemeAwareValue` | `md` | Button size. The icon takes 55% of it. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. |
| `themes` | `&'static [&'static ThemeSet]` | - | Adds the theme picker, a second button that opens a menu of these sets. `class`, `sx` and extra attributes then land on the group around both. |
| `with_system` | `bool` | `false` | Adds following the system to the cycle. Off, a press flips between light and dark, and flipping to the system's own scheme follows the system again. |
| `label` | `Callback<ColorSchemeSetting, String>` | - | Replaces the three built-in button names. Gets the setting a press moves to and returns what the press does. |
| `disabled` | `bool` | `false` | Disables and dims the button. |
| `parts` | `Parts<ThemeTogglePart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `ThemeToggle` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

## Style API

Without `themes` the toggle is the root itself, styled by `sx`; `Toggle`, `Picker` and `Chevron` exist only with `themes`. The theme-set menu is portaled and out of reach.

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ThemeTogglePart::Icon` | `icon` | The sun, moon or system glyph. |
| `ThemeTogglePart::Toggle` | `toggle` | The scheme button beside the picker. |
| `ThemeTogglePart::Picker` | `picker` | The button that opens the theme-set menu. |
| `ThemeTogglePart::Chevron` | `chevron` | The picker's chevron. |

## Accessibility

### Libero handles

- The button's name says what a press does, from the localization's
  `ThemeToggleLabels`: `to_light` and `to_dark`, plus `to_system` with `with_system`.
- With `themes`, both buttons sit in a `role="group"` named by `group`. The
  picker is named by `picker` and opens a [`Menu`](menu.md), with its keys; the
  sets are radio items in a group named by `themes`.

### You must

- With `label`, return what the press does, not the current scheme.

## Theme defaults

`ThemeToggleDefaults` on the theme, as `theme_toggle`.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`outlined`). |
| `color` | `Color` | Default `color` when the prop is omitted (`muted`). |

The names are `ThemeToggleLabels` in the localization: `to_light`,
`to_dark`, `to_system` (the toggle's names), `group`, `picker` and `themes`
(the picker's).

## CSS variables

None of its own; it renders an `ActionIcon`, whose variables apply.

## Data attributes

`ActionIcon`'s, on the root's `data-state`.
