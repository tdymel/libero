# Text

Crate: `libero`
Import: `use libero::components::Text;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/text.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Body copy, sized from the theme's text scale.

Body copy - renders a `<p>` by default, sized via the theme's text scale.
`size` picks a step of that scale (font size, weight, letter spacing and line
height together), and `component` changes the element without changing the
look. For headings use [title.md](title.md) instead - a `Title`'s size also
picks its heading level.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Text;

#[component]
fn Demo() -> Element {
    rsx! {
        Text { size: "md", component: "p", "The quick brown fox jumps over the lazy dog." }
    }
}
```

## Accessibility

`Text` renders a paragraph by default, so it is read as one block. `component:
"span"` or `"div"` drops that grouping - use `span` for text inside a sentence
and keep `p` for standalone copy. Size is styling only; it never changes the
element, so it cannot be used to imply a heading.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Visual size, `xs` through `xxl`. |
| `component` | `HtmlTag` | `p` | Which element to render as. |
| `children` | `Element` | required | The text content. |

Like every component, `Text` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`TextDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `font_family` | `&'static str` | Font stack for all text; it does not vary by size. |
| `sizes` | `Sizes<TextSize>` | `font_weight`, `font_size`, `letter_spacing`, `line_height` per size. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-text-font-family` | Font stack, shared by every size. |
| `--lsx-text-font-size-<size>` | `font-size` for that size step. |
| `--lsx-text-font-weight-<size>` | `font-weight` for that size step. |
| `--lsx-text-letter-spacing-<size>` | `letter-spacing` for that size step. |
| `--lsx-text-line-height-<size>` | `line-height` for that size step. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect - this is what selects the size variables. |
