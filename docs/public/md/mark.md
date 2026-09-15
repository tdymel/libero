# Mark

Crate: `libero`
Import: `use libero::components::Mark;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/mark.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A real `mark` element that highlights a chunk of text with a light tint of a theme color.

Highlight a chunk of the text. Renders a real `<mark>`, tinted with a light shade
of the theme's `warning` color by default.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Mark, Text};

#[component]
fn Demo() -> Element {
    rsx! {
        Text {
            "Highlight "
            Mark { color: "warning", "this chunk" }
            " of the text."
        }
    }
}
```

A bare theme color name is tinted to a light shade; an explicit shade
(`"error.4"`) keeps its shade. A theme color paints its fill shade, the one its
contrast color is computed on, and sets the text to that contrast color. A hex
sets black or white text; any other literal CSS color (`"gold"`) passes through
and the text inherits.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `warning`, tinted | Any theme color or literal value; a bare theme color is tinted to a light shade. A theme color paints its fill shade and sets the text to its contrast twin; a hex sets black or white text. |
| `children` | `Element` | required | The highlighted content. |

Like every component, `Mark` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`MarkDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `color` | `Color` | Which theme color the tint is taken from; the shade is fixed at 1. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-mark-background` | The resolved tint, set per instance from `color` or the theme default. |
| `--lsx-mark-color` | The tint's contrast color for the text, set per instance. Unset for a literal other than a hex. |
| `--lsx-focus-contrast` | The same contrast color, published for focus rings inside the mark. |
| `--lsx-focus-ring-halo` | The tint, published beside it as the ring's halo, so a dark shade's white ring still reads. |

## Data attributes

Only what you pass: the `states` prop renders as `data-state`. `Mark` adds no
state tokens of its own.
