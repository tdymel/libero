# ColorSchemeButton

Crate: `libero`
Import: `use libero::components::ColorSchemeButton;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/color_scheme_button.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An icon button that steps the colour scheme through system, dark and light, with an opt-in theme-set picker beside it.

An icon button that steps the app's colour scheme: following the platform,
then the scheme the platform is not showing, then the one it is, then back to
following it. The glyph and the accessible name both say where a press goes:
a sun switches to light, a moon to dark, a half-filled disc back to following
the platform. It is an
`ActionIcon` over `use_color_scheme()`.

While it follows the platform, an OS switch - or a devtools emulation of
`prefers-color-scheme` - is followed live. A pinned scheme stays pinned until a
press hands the choice back.

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

With `themes` it becomes a split button: a chevron beside the toggle opens a
menu of theme sets, the active one checked. Two buttons rather than one with a
second gesture, so each has one job, one name and its own tab stop.

```rust
use dioxus::prelude::*;
use libero::{components::ColorSchemeButton, theme::ThemeSet};

#[component]
fn Demo() -> Element {
    rsx! {
        ColorSchemeButton { size: "lg", themes: ThemeSet::CATALOGUE }
    }
}
```

The names come from the [localization](localization.md), so other
wording is one struct:

```rust
use libero::localization::{ColorSchemeButtonLabels, Localization};

static WORDS: Localization = Localization {
    color_scheme_button: ColorSchemeButtonLabels {
        to_light: "Helles Design",
        to_dark: "Dunkles Design",
        to_system: "Wie das System",
        ..ColorSchemeButtonLabels::GERMAN
    },
    ..Localization::GERMAN
};
```

An app that wants an explicit "follow the system" choice builds it from
`use_color_scheme()` instead - see [Theming](theming.md).

## Accessibility

The accessible name says what a press does - "Switch to the dark theme",
"Follow the system theme" - the same thing the `aria-hidden` glyph shows.
Opening the picker puts focus on the checked theme set, not the first one.

With `themes`, the pair is a `role="group"` named "Theme". The chevron is its
own button, "Choose a theme", with `aria-haspopup="menu"` and `aria-expanded`;
the sets in its menu are `menuitemradio`s with `aria-checked`. It is never
narrower than 24px (WCAG 2.5.8).

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | `outlined` | `ActionIcon`'s chrome. Unset, the theme's `color_scheme_button.variant`. |
| `color` | `ThemeAwareValue` | `muted` | Accent color. Unset, the theme's `color_scheme_button.color`. |
| `size` | `ThemeAwareValue` | `md` | Button size; the glyph takes 55% of it. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of size. |
| `themes` | `&'static [&'static ThemeSet]` | - | Opts into the theme picker: a chevron beside the toggle opening a menu of these sets, the active one checked. The pair is then a named `group`, and `class`, `sx` and extra attributes land on it. |
| `label` | `Callback<ColorSchemeSetting, String>` | - | Replaces the localization's three toggle names. Given the setting a press moves to, it names what the press does. |
| `disabled` | `bool` | `false` | Disables interaction and dims the button. |

Like every component, `ColorSchemeButton` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

## Theme defaults

`ColorSchemeButtonDefaults` on the theme, as `color_scheme_button`.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`outlined`). |
| `color` | `Color` | Default `color` when the prop is omitted (`muted`). |

The names are `ColorSchemeButtonLabels` in the localization: `to_light`,
`to_dark`, `to_system` (the toggle's names), `group`, `picker` and `themes`
(the picker's).

## CSS variables

None of its own; it renders an `ActionIcon`, whose variables apply.

## Data attributes

`ActionIcon`'s, on the root's `data-state`.
