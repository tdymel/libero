# Title

Crate: `libero`
Import: `use libero::components::Title;`
Feature: `title` (on by default, turns on `box`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/title.rs>
Index: [index.md](index.md) lists every other page
Description: A heading, `h1` through `h6`, whose visual size and semantic tag can be set apart.

A heading, `h1` to `h6`. `size` sets the look and the tag: `xxl` is `h1`, `xl`
is `h2`, down to `xs` as `h6`. Set `component` when the look and the level
disagree.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::Title, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Title {
            component: "h2",
            truncate: "end",
            title: "The quick brown fox jumps over the lazy dog and keeps running",
            sx: sx().max_width("20rem"),
            "The quick brown fox jumps over the lazy dog and keeps running"
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `xxl` | Visual size, `xs` to `xxl`. Also picks the tag unless `component` is set. |
| `component` | `HtmlTag` | follows `size` | The heading tag. The size's look stays. With `size` unset it is `h1`, whatever the theme's size. A debug build warns for a tag other than `h1` to `h6`. |
| `gradient` | `Gradient` | - | Paints the glyphs with a gradient from the theme's first stop to a second, as `("info", 90)` or `Gradient::default().to("info").deg(90)`; `Gradient::default()` is the theme's. The contrast of a literal CSS stop is yours to check, and a debug build warns when a hex stop reads under 4.5:1 on the page background. Solid in its first stop in print and in native windows, the system text color in forced colors. |
| `truncate` | `Truncate` | - | Cuts the title on one line with an ellipsis at its `end` or `start`. `line_clamp` wins when both are set. |
| `line_clamp` | `u32` | - | Cuts the title after this many lines with an ellipsis. Over `truncate`; `0` is off. On Blitz there is no line clamp: the title is cut at that many lines, without the ellipsis. |
| `children` | `Element` | required | The heading text. |

Like every component, `Title` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- `size` picks the heading tag, `xxl` as `h1` down to `xs` as `h6`, unless
  `component` is set.
- A cut heading is still whole in the DOM, so a screen reader and the outline
  read all of it.

### You must

- Pass a heading that may be cut as `title` too, so a mouse user can read it on
  hover. A hover title does not reach touch or keyboard users: on narrow screens
  prefer `line_clamp` or let the heading wrap, and show the full text elsewhere
  where it matters.
- Keep one `h1` per page and skip no levels.
- A `lg` heading in a section under the page's `h1` needs `component: "h2"`, or
  the document jumps from `h1` to `h3`.
- Check the contrast of a literal CSS stop in `gradient`. A debug build warns
  when a hex stop reads under 4.5:1 on the page background.

### Example

A section heading under the page's `h1`, `Title { size: "lg", component: "h2",
"Pricing" }`: a screen reader lists "Pricing" as a level 2 heading, whatever
its size.

### Limits

- On Blitz, `line_clamp` cuts the title at its last line without an ellipsis.

## Theme defaults

`TitleDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `xxl`. The look only: a bare `Title` stays `h1` whatever this says. |
| `font_family` | `&'static str` | Heading font stack; the theme's sans stack. It does not vary by size. |
| `sizes` | `Sizes<TitleSizeLevel>` | `font_weight`, `font_size` (rem), `letter_spacing`, `line_height` per size. |

The default ramp runs 0.75rem (`xs`) to 2.125rem (`xxl`), with `letter_spacing`
tightening to `-0.01em` and `line_height` to `1.3` at the top.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-title-font-family` | Heading font stack; one value for every size. |
| `--lsx-title-font-size-<size>` | `font-size` for that size step. |
| `--lsx-title-font-weight-<size>` | `font-weight` for that size step. |
| `--lsx-title-letter-spacing-<size>` | `letter-spacing` for that size step. |
| `--lsx-title-line-height-<size>` | `line-height` for that size step. |

Every size is emitted at once behind its own `data-state` selector, so all
`Title`s share one class and picking a size never mints a new one.

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect, whatever tag `component` chose. |
| `truncate-end` / `truncate-start` | The `truncate` in effect, without a `line_clamp`. |
| `line-clamp` | `line_clamp` is above 0. |
