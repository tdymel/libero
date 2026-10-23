# Text

Crate: `libero`
Import: `use libero::components::Text;`
Feature: `text` (on by default, turns on `box`; [features.md](features.md))
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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Visual size, `xs` to `xxl`. |
| `component` | `HtmlTag` | `p` | The element to render. |
| `color` | `ThemeAwareValue` | - | The text color: a theme color name in its text shade, or any CSS color. Unset, the text inherits. Under a gradient, its first stop. |
| `gradient` | `Gradient` | - | Paints the glyphs with a gradient from `color` to a second stop, as `("info", 90)` or `Gradient::default().to("info").deg(90)`; `Gradient::default()` is the theme's. Keep it to large display text: the contrast of a literal CSS stop is yours to check, and a debug build warns when a hex stop reads under 4.5:1 on the page background. Solid in its first stop in print and in native windows, the system text colour in forced colours. |
| `truncate` | `Truncate` | - | Cuts the text on one line with an ellipsis at its `end` or `start`. The element becomes a block, so a `span` truncates too. Set a `title` if the cut text matters; the full text stays in the accessible name. `line_clamp` wins when both are set. |
| `line_clamp` | `u32` | - | Cuts the text after this many lines with an ellipsis. Over `truncate`; `0` is off. On Blitz there is no line clamp: the text is cut at that many lines, without the ellipsis. |
| `children` | `Element` | required | The text. |

Like every component, `Text` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- A large size is only styling, so it never makes a heading.
- A long word breaks inside the text rather than overflowing a narrow column.
- A cut text is still whole in the DOM, so a screen reader reads all of it.

### You must

- Use `component: "span"` for text inside a sentence.
- For a heading, use [`Title`](title.md).
- Check the contrast of a literal CSS `color` on its background, 4.5:1 for body
  text: a theme color name takes its text shade, a literal is used as given.
- Keep `gradient` to large display text and check the contrast of a literal CSS
  stop. A debug build warns when a hex stop reads under 4.5:1 on the page
  background.
- Pass a text that may be cut as `title` too, so sighted users can read it on
  hover, or show it in full elsewhere.

### Example

A price in a sentence, `Text { component: "span", size: "lg", "$12" }`: it
stays part of the sentence, and its large size does not make it a heading.

### Limits

- `component` keeps the body-text look, so `a`, `strong`, `del` and the like
  lose their underline, weight or line. For a link, use [`Anchor`](anchor.md).
- On Blitz, `line_clamp` cuts the text at its last line without an ellipsis.

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
| `truncate-end` / `truncate-start` | The `truncate` in effect, without a `line_clamp`. |
| `line-clamp` | `line_clamp` is above 0. |
