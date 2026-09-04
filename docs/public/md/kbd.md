# Kbd

Crate: `libero`
Import: `use libero::components::Kbd;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/kbd.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A single keyboard key, rendered as a real `<kbd>` and styled entirely from the theme.

Renders a real `<kbd>` - a keycap, with a slightly thicker bottom border so it
reads as having depth rather than as a flat pill. Styled entirely from the theme
(`Theme::kbd`); `size` is the only prop.

## Usage

One `Kbd` is one key. A shortcut is several of them with your own separator, so
the sentence reads the way you want it to.

```rust
use dioxus::prelude::*;
use libero::components::{Kbd, Text};

#[component]
fn Demo() -> Element {
    rsx! {
        Text {
            "Save with "
            Kbd { "Ctrl" }
            " + "
            Kbd { "S" }
            "."
        }
    }
}
```

`size` steps the font size; every other part of the look is the theme's.

```rust
use dioxus::prelude::*;
use libero::components::Kbd;

#[component]
fn Demo() -> Element {
    rsx! {
        Kbd { size: "lg", "Ctrl" }
    }
}
```

## Accessibility

Spell the key the way the platform labels it and put the separator in the
surrounding text - a screen reader reads `Kbd { "Ctrl" } " + " Kbd { "S" }` as
"Ctrl plus S", where one `Kbd { "Ctrl+S" }` reads as a single opaque token.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `sm` | Font size. Everything else about the look is `Theme::kbd` only. |
| `children` | `Element` | required | The key label. |

Like every component, `Kbd` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`KbdDefaults` on the theme. The default `size` is not a theme field - `Kbd`
hardcodes `sm`, matching Mantine.

| Field | Type | Description |
|---|---|---|
| `font_size` | `Sizes<u16>` | Font size in px per size step - 10, 12, 14, 16, 20, 24. |
| `font_family` | `&'static str` | The keycap's font; the theme's mono stack. |
| `background` | `&'static str` | Keycap fill; `#f6f8fa`. |
| `border` | `&'static str` | Border color, used on all four sides; `#d0d7de`. |
| `color` | `&'static str` | Label color; `#57606a`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-kbd-font-size-<size>` | `font-size` for that size step. |
| `--lsx-kbd-font-family` | The keycap's font family. |
| `--lsx-kbd-background` | Keycap fill. |
| `--lsx-kbd-border` | Border color; the bottom border draws it 3px, the others 1px. |
| `--lsx-kbd-color` | Label color. |

The corner radius reads the shared `--lsx-radius-sm`.

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
