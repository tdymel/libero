# Theming

Crate: `libero`
Import: `use libero::{LiberoProvider, theme::{HexColor, Size, Sizes, Theme}};`
Index: [index.md](index.md) lists every other page
Description: How to customize a Libero theme and use it: colors, scales, per-component defaults, light and dark pairs, and reading the active theme.

A theme is one struct. Hand yours to `LiberoProvider` and every component
follows it.

| Part | Fields | What it changes |
|---|---|---|
| Colors | `primary`, `secondary`, `success`, `error`, `warning`, `info` | One hex per role, with nine shades and a text color derived from it. |
| Page | `surface`, `ink` | The page and the text on it. A dark surface makes a dark theme. |
| Scales | `spacing`, `radius`, `elevation` | Every size word, gap, corner and shadow. |
| Components | `button`, `dialog`, ... | Every prop a caller leaves unset. |

## A custom theme

Name the fields you change. The rest come from `Theme::DEFAULT`.

```rust,ignore
static THEME: Theme = Theme {
    primary: HexColor::new(0x7C3AED),
    spacing: Sizes::new(4, 8, 12, 16, 20, 24),
    ..Theme::DEFAULT
};

fn App() -> Element {
    rsx! {
        LiberoProvider {
            themes: &THEME,
            Router::<Route> {}
        }
    }
}
```

## Colors

One hex per role is enough. Libero derives nine shades from it and keeps text
legible on each. In `sx`, a role name is its base color and `primary.1` to
`primary.9` are the shades.

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Text}, sx::sx};

#[component]
fn Status() -> Element {
    rsx! {
        Text { sx: sx().color("error"), "Payment failed" }
        Box { sx: sx().background("primary.1").padding("md").border_radius("md"), "Saved" }
    }
}
```

## Component defaults

Every prop a caller leaves unset comes from the component's struct on the theme.
Pill-shaped buttons everywhere is one change here.

```rust,ignore
static THEME: Theme = Theme {
    button: ButtonDefaults {
        radius: Size::Xl,
        ..ButtonDefaults::DEFAULT
    },
    ..Theme::DEFAULT
};
```

## Light and dark

A `ThemeSet` pairs a light theme with a dark one. The default pairs
`Theme::DEFAULT` with `Theme::DARK` and follows the system setting. A dark theme
is one whose `surface` is dark and `ink` light. The roles adapt on their own.
`ThemeSet::CATALOGUE` has ready-made pairs such as Catppuccin, Dracula and Nord.

```rust,ignore
LiberoProvider {
    themes: ThemeSet::new().light(&LIGHT).dark(&DARK),
    Router::<Route> {}
}
```

[`ThemeToggle`](theme_toggle.md) is a ready-made switch.

On the web the choice is kept in `localStorage`. To restore it before the first
paint, paste `libero::theme::COLOR_SCHEME_RESTORE_SCRIPT` into the head of your
`index.html`.

## Reading the theme

`use_theme()` returns the active theme, for a value you need in Rust rather than
in `sx`.

```rust
use dioxus::prelude::*;
use libero::{components::Text, theme::Size, use_theme};

#[component]
fn Gap() -> Element {
    let theme = use_theme();
    let gap = theme.spacing.get(Size::Md);
    rsx! { Text { "The md gap is {gap}px" } }
}
```

## Theme fields

`Theme` is about 2 KB and not `Copy`, so a stray by-value use cannot copy it
silently. `use_theme()` hands out a `&'static Theme`.

| Field | Type | Description |
|---|---|---|
| `spacing` | `Sizes<u8>` | Pixels per spacing step, xs to xxl. Every size word, gap and padding steps along it. |
| `radius` | `Sizes<u8>` | Pixels per radius step. |
| `elevation` | `Sizes<&'static str>` | One box-shadow per elevation step. `Theme::DARK` has a deeper scale, since a shadow on a dark page needs more alpha to show. |
| `primary`, `secondary`, `success`, `error`, `warning`, `info`, `neutral`, `muted` | `HexColor` | One hex per palette role. Nine shades, and a readable text color for each, are derived from it. |
| `ink`, `surface` | `HexColor` | The text color and the page it is set on. Every role is derived against `surface`. |
| `font_smoothing` | `bool` | Whether the reset asks for antialiased text. |
| one field per component | `*Defaults` | For example `button: ButtonDefaults`. Each component's page lists its own struct. |

## CSS variables

Everything the theme emits lives under the `--lsx-` prefix on `:root`.

| Variable | Description |
|---|---|
| `--lsx-<role>-<1..9>` | A shade of a palette role; shade 6 is the brand color itself. |
| `--lsx-<role>-text-<1..9>` | The same ramp re-based on the first shade that reads on the page; what `color()` resolves a palette color through. |
| `--lsx-<role>-fill-<1..9>` | The same ramp re-based on the first shade that carries a black or white foreground; what `background()` resolves through. |
| `--lsx-<role>-contrast-<1..9>` | Black or white, whichever reads on that step of the fill ramp. |
| `--lsx-text-dimmed` | Secondary text: a placeholder, a hint, a unit. Written `"text-dimmed"` in an `sx`. |
| `--lsx-spacing-<size>` | A step of the spacing scale. |
| `--lsx-radius-<size>` | A step of the radius scale. |
| `--lsx-shadow-<size>` | A step of the elevation scale. |
| `--lsx-<component>-*` | A component's own defaults; see that component's page. |
