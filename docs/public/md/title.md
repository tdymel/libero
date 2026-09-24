# Title

Crate: `libero`
Import: `use libero::components::Title;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/title.rs>
Index: [index.md](index.md) lists every other page
Description: A heading, `h1` through `h6`, whose visual size and semantic tag can be set apart.

A heading, `h1` to `h6`. `size` sets the look and the tag: `xxl` is `h1`, `xl`
is `h2`, down to `xs` as `h6`. Set `component` when the look and the level
disagree.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Title;

#[component]
fn Demo() -> Element {
    rsx! {
        Title { "The quick brown fox" }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `xxl` | Visual size, `xs` to `xxl`. Also picks the tag unless `component` is set. |
| `component` | `HtmlTag` | follows `size` | The heading tag. The size's look stays. |
| `children` | `Element` | required | The heading text. |

Like every component, `Title` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- `size` picks the heading tag, `xxl` as `h1` down to `xs` as `h6`, unless
  `component` is set.

### You must

- Keep one `h1` per page and skip no levels.
- A `lg` heading in a section under the page's `h1` needs `component: "h2"`, or
  the document jumps from `h1` to `h3`.

```rust
use dioxus::prelude::*;
use libero::components::Title;

#[component]
fn Demo() -> Element {
    rsx! {
        Title { size: "lg", component: "h2", "The quick brown fox" }
    }
}
```

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
