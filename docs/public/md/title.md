# Title

Crate: `libero`
Import: `use libero::components::Title;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/title.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A heading, `h1` through `h6`, whose visual size and semantic tag can be set apart.

A heading, `h1` through `h6` - `component` decouples the semantic tag from the
visual size, for a11y heading order.

`size` picks the tag on its own: `xxl` is `h1`, `xl` is `h2`, `lg` is `h3`, `md` is
`h4`, `sm` is `h5`, `xs` is `h6`. So a page's headings come out in order as long as
the sizes descend.

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

`size` is both the look and the level:

```rust
use dioxus::prelude::*;
use libero::components::Title;

#[component]
fn Demo() -> Element {
    rsx! {
        Title { size: "lg", "The quick brown fox" }
    }
}
```

## Heading order

`component` overrides the tag while keeping the size's visual weight, which is how
`h1 -> h2 -> h3` order survives a layout that wants a big heading somewhere deep.
Set it whenever a `Title` is not at the page's top level: a `lg` heading inside a
section under the page's `h1` needs `component: "h2"`, or the document skips from
`h1` straight to `h3`.

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

## Accessibility

One `h1` per page, and no skipped levels - which is what `component` is for,
since the level a layout wants and the size a design wants often disagree.
`Title` will not stop you from emitting a broken order.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `xxl` | Visual size, `xs` through `xxl`. Also picks the heading tag. |
| `component` | `HtmlTag` | follows `size` | Overrides the heading tag, keeping a size's weight under a different level so `h1 -> h2 -> h3` order survives. |
| `children` | `Element` | required | The heading text. |

Like every component, `Title` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`TitleDefaults` on the theme; the default `size` is hardcoded to `xxl` rather than
being a theme field.

| Field | Type | Description |
|---|---|---|
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
| `size-<size>` | The `size` in effect - independent of which tag `component` chose. |
