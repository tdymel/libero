# Kbd

Crate: `libero`
Import: `use libero::components::Kbd;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/kbd.rs>
Index: [index.md](index.md) lists every other page
Description: A single keyboard key, rendered as a real `<kbd>` and styled from the theme.

One keyboard key in a real `<kbd>`, styled from the theme. A shortcut is
several keys with your own separator.

## Usage

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

## Accessibility

### Libero handles

- Each key is a real `<kbd>`.

### You must

- Put the separator in the text around the keys. A screen reader reads
  `Kbd { "Ctrl" } " + " Kbd { "S" }` as "Ctrl plus S", but one
  `Kbd { "Ctrl+S" }` as a single token.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `sm` | Font size. The rest of the look comes from the theme. |
| `children` | `Element` | required | The key label. |

Like every component, `Kbd` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`KbdDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `sm`. |
| `font_sizes` | `Sizes<u16>` | Font size in px per size step: 10, 12, 14, 16, 20, 24. |
| `font_family` | `&'static str` | The keycap's font; the theme's mono stack. |
| `background` | `&'static str` | Keycap fill; `var(--lsx-muted-1)`. |
| `border` | `&'static str` | Border color, used on all four sides; `var(--lsx-muted-4)`. |
| `color` | `&'static str` | Label color; `var(--lsx-muted-7)`, darkened or lightened until it reads at 4.5:1. |

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
