# ColorSchemeButton

Crate: `libero`
Import: `use libero::components::ColorSchemeButton;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/color_scheme_button.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An icon button that flips the app between its light and dark theme - an `ActionIcon` over `use_color_scheme()`.

An icon button that flips the app between its light and dark theme: a moon
while the light scheme shows, a sun while the dark one does. It is an
`ActionIcon` over `use_color_scheme()`, named for what a press does.

A press pins the other scheme only while it differs from the platform's.
Flipping back hands the choice to the platform again, so an OS switch - or a
devtools emulation of one - is followed from then on. The platform coming
round to a pinned scheme drops the pin too.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::ColorSchemeButton;

#[component]
fn Demo() -> Element {
    rsx! {
        ColorSchemeButton { size: "lg" }
    }
}
```

The two names come from the theme, so a translation is one struct:

```rust
use libero::theme::{ColorSchemeButtonDefaults, ColorSchemeButtonLabels, Theme};

static GERMAN: Theme = Theme {
    color_scheme_button: ColorSchemeButtonDefaults {
        labels: ColorSchemeButtonLabels {
            to_light: "Helles Design",
            to_dark: "Dunkles Design",
        },
        ..ColorSchemeButtonDefaults::DEFAULT
    },
    ..Theme::DEFAULT
};
```

An app that wants an explicit "follow the system" choice builds it from
`use_color_scheme()` instead - see [Theming](theming.md).

## Accessibility

The accessible name says what a press does - "Switch to the dark theme" - not
which scheme is showing: a screen reader user cannot see the glyph it swaps.
The glyph is `aria-hidden`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | `outlined` | `ActionIcon`'s chrome. Unset, the theme's `color_scheme_button.variant`. |
| `color` | `ThemeAwareValue` | `muted` | Accent color. Unset, the theme's `color_scheme_button.color`. |
| `size` | `ThemeAwareValue` | `md` | Button size; the glyph takes 55% of it. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of size. |
| `label` | `Callback<ColorScheme, String>` | - | Replaces the theme's two names. Given the scheme on screen, it names what a press does. |
| `disabled` | `bool` | `false` | Disables interaction and dims the button. |

Like every component, `ColorSchemeButton` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

## Theme defaults

`ColorSchemeButtonDefaults` on the theme, as `color_scheme_button`.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`outlined`). |
| `color` | `Color` | Default `color` when the prop is omitted (`muted`). |
| `labels` | `ColorSchemeButtonLabels` | `to_light` and `to_dark`, the two accessible names. |

## CSS variables

None of its own; it renders an `ActionIcon`, whose variables apply.

## Data attributes

`ActionIcon`'s, on the root's `data-state`.
