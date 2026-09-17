# Mark

Crate: `libero`
Import: `use libero::components::Mark;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/mark.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A `mark` element that highlights text with a light tint of a theme color.

Highlights a chunk of text in a real `<mark>`, tinted with a light shade of the
theme's `warning` color by default. The text color follows the tint, so it
stays readable.

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

A link inside a `Mark` takes the same contrast color as the text, since no one
link color reads on every tint, and is always underlined so it still stands out.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `warning`, tinted | A theme color name gets a light shade, and an explicit shade such as `error.4` stays as it is. Any CSS color works too. |
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
| `--lsx-anchor-color` | The same contrast color again, for an `Anchor` inside the mark. |
| `--lsx-focus-ring-halo` | The tint, published beside it as the ring's halo, so a dark shade's white ring still reads. |

## Data attributes

Only what you pass: the `states` prop renders as `data-state`. `Mark` adds no
state tokens of its own.
