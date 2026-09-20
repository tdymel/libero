# Text

Crate: `libero`
Import: `use libero::components::Text;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/text.rs>
Index: [index.md](index.md) lists every other page
Description: Body copy, sized from the theme's text scale.

Body copy in a `<p>`, sized from the theme's text scale. `component` changes
the element without changing the look. For headings, use [Title](title.md).

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

### Libero handles

- A large size is only styling, so it never makes a heading.

### You must

- Use `component: "span"` for text inside a sentence.
- For a heading, use [`Title`](title.md).
- Keep `gradient` to large display text and check the contrast of a literal CSS
  stop. A debug build warns when a hex stop reads under 4.5:1 on the page
  background.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Visual size, `xs` to `xxl`. |
| `component` | `HtmlTag` | `p` | The element to render. |
| `children` | `Element` | required | The text. |

Like every component, `Text` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`TextDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `font_family` | `&'static str` | Font stack for all text; it does not vary by size. |
| `sizes` | `Sizes<TextSizeLevel>` | `font_weight`, `font_size`, `letter_spacing`, `line_height` per size. |

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
| `size-<size>` | The `size` in effect. |
