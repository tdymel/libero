# ThemeToggle

Crate: `libero`
Import: `use libero::components::ThemeToggle;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/theme_toggle.rs>
Index: [index.md](index.md) lists every other page
Description: An icon button that steps the colour scheme through system, dark and light, with an optional theme picker beside it.

An icon button that switches the app's colour scheme. Each press steps from
following the system, to the scheme the system is not showing, to the one it
is, and back. The icon shows where the next press goes: a sun for light, a moon
for dark, a half-filled disc for following the system.

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
        ThemeToggle { size: "md", themes: ThemeSet::CATALOGUE }
    }
}
```

## Accessibility

### Libero handles

- The button's name says what a press does, from the localization's
  `ThemeToggleLabels`: `to_light`, `to_dark` or `to_system`.
- With `themes`, both buttons sit in a `role="group"` named by `group`. The
  picker is named by `picker` and opens a [`Menu`](menu.md), with its keys; the
  sets are radio items in a group named by `themes`.

### You must

- With `label`, return what the press does, not the current scheme.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | `outlined` | Visual style, as on `ActionIcon`. |
| `color` | `ThemeAwareValue` | `muted` | Accent color. A theme color name or any CSS color. |
| `size` | `ThemeAwareValue` | `md` | Button size. The icon takes 55% of it. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. |
| `themes` | `&'static [&'static ThemeSet]` | - | Adds the theme picker, a second button that opens a menu of these sets. `class`, `sx` and extra attributes then land on the group around both. |
| `label` | `Callback<ColorSchemeSetting, String>` | - | Replaces the three built-in button names. Gets the setting a press moves to and returns what the press does. |
| `disabled` | `bool` | `false` | Disables and dims the button. |

Like every component, `ThemeToggle` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

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
