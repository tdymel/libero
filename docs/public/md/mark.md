# Mark

Crate: `libero`
Import: `use libero::components::Mark;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/mark.rs>
Index: [index.md](index.md) lists every other page
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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `warning`, tinted | A theme color name gets a light shade, and an explicit shade such as `error.4` stays as it is. Any CSS color works too. Under a gradient, its first stop. |
| `gradient` | `Gradient` | - | Fills the highlight with a gradient from `color` to a second stop, as `("info", 90)` or `Gradient::default().to("info").deg(90)`; `Gradient::default()` is the theme's. The text turns black or white, whichever reads on both stops and the span between; the contrast of a literal CSS stop is yours to check. Solid in its first stop where the image is dropped. |
| `children` | `Element` | required | The highlighted content. |

Like every component, `Mark` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- Each highlight is a real `<mark>`.
- For a theme color, a shade or a hex, the text takes the tint's contrast
  color, so it stays readable.
- A link inside is underlined in the text's color, unless its `underline` is
  `never`, and its focus ring clears 3:1 against the tint.
- In forced colors the tint gives way to the system highlight colors, `Mark`
  and `MarkText`.
- Under a `gradient`, the text is black or white, picked to read at 4.5:1 on
  both stops and the span between in light and dark.

### You must

- With a CSS color name such as `gold`, the text keeps the page's color: check
  its contrast.
- Check the contrast of a literal CSS stop in `gradient`.
- Say in the text why a highlight matters. Not every screen reader announces
  `<mark>`.

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
