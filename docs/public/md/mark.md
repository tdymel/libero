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
(`"error.4"`) or a literal CSS color (`"gold"`) passes through untouched.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `warning`, tinted | Any theme color or literal value; a bare theme color is tinted to a light shade. |
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

## Data attributes

Only what you pass: the `states` prop renders as `data-state`. `Mark` adds no
state tokens of its own.
