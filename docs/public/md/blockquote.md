# Blockquote

Crate: `libero`
Import: `use libero::components::Blockquote;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/blockquote.rs>
Index: [index.md](index.md) lists every other page
Description: A quotation in a tinted frame with an accent bar, with the attribution outside the quote.

A quotation in a tinted frame with an accent bar. The attribution sits in a
`<figcaption>` outside the `<blockquote>`, so a screen reader does not read the
speaker's name as part of the quote. `size` scales the text with the frame, on
[Text](text.md)'s scale. The accent bar is on the left in every writing
direction.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Blockquote;

#[component]
fn Demo() -> Element {
    rsx! {
        Blockquote {
            attribution: rsx! { "Albert Einstein" },
            "Life is like riding a bicycle. To keep your balance, you must keep moving."
        }
    }
}
```

With every prop set:

```rust
use dioxus::prelude::*;
use libero::components::Blockquote;

#[component]
fn Demo() -> Element {
    rsx! {
        Blockquote {
            size: "lg",
            color: "info",
            radius: "md",
            attribution: rsx! { "Albert Einstein" },
            work: "Letter to his son Eduard",
            cite_url: "https://example.org/letters/1930-02-05",
            "Life is like riding a bicycle. To keep your balance, you must keep moving."
        }
    }
}
```

## Accessibility

### Libero handles

- The quote is a `<blockquote>` in a `<figure>`, and the attribution sits in a
  `<figcaption>` outside it, so a screen reader does not read the speaker's name
  as part of the quote.
- `work` renders in a `<cite>`, the comma kept outside it.

### Limits

- `cite_url` is for machines only: browsers do not show it, so link the source
  yourself where readers need it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Text size, line height, padding and the accent bar's width. |
| `color` | `ThemeAwareValue` | `primary`, tinted | The accent bar and the tint behind the quote. A theme color name gets its lightest shade. Any CSS color works too. |
| `radius` | `Size` | `sm` | Rounds the two corners away from the accent bar. |
| `attribution` | `Element` | - | Who said it, shown under the quote. For any join other than a comma, pass the whole line here. |
| `work` | `String` | - | The title of the quoted work, such as a book or a talk. Follows `attribution` after a comma. |
| `cite_url` | `String` | - | A URL naming the source. Only machines read it, browsers do not show it. |
| `children` | `Element` | required | The quote. |

Like every component, `Blockquote` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the `<figure>`.

## Theme defaults

`BlockquoteDefaults` on the theme, as `theme.blockquote`.

| Field | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `Size::Md` | The size used when the prop is unset. |
| `radius` | `Size` | `Size::Sm` | The radius used when the prop is unset. |
| `color` | `Color` | `Color::Primary` | Which theme color the tint and the bar are taken from. |
| `cite_opacity` | `&'static str` | `"0.65"` | How far the `<figcaption>` is dimmed below the quote. |
| `sizes` | `Sizes<BlockquoteSizeLevel>` | below | Padding and bar width per size. |

`BlockquoteSizeLevel`, per size:

| Size | `padding_y` | `padding_x` | `border_width` |
|---|---|---|---|
| `xs` | `0.5rem` | `0.75rem` | `2px` |
| `sm` | `0.75rem` | `1rem` | `2px` |
| `md` | `1rem` | `1.5rem` | `3px` |
| `lg` | `1.25rem` | `2rem` | `3px` |
| `xl` | `1.5rem` | `2.5rem` | `4px` |
| `xxl` | `2rem` | `3rem` | `5px` |

The font size and line height come from `TextDefaults`' scale, so retuning
`theme.text` retunes quotes too. The `<figcaption>` is 0.85 of that font size.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-blockquote-padding-y-{xs..xxl}` | Vertical padding per size. Declared on `:root` from the theme. |
| `--lsx-blockquote-padding-x-{xs..xxl}` | Horizontal padding per size. Declared on `:root` from the theme. |
| `--lsx-blockquote-border-width-{xs..xxl}` | The accent bar's width per size. Declared on `:root` from the theme. |
| `--lsx-blockquote-cite-opacity` | The `<figcaption>`'s opacity. Declared on `:root` from the theme. |
| `--lsx-blockquote-background` | The resolved tint, set per instance on the `<blockquote>`. |
| `--lsx-blockquote-border-color` | The resolved accent bar color, set per instance on the `<blockquote>`. |
| `--lsx-blockquote-color` | The tint's contrast color for the body text, set per instance. Unset for a literal other than a hex. |
| `--lsx-focus-contrast` | The same contrast color, published for focus rings inside the quote. Unset for a literal other than a hex. |
| `--lsx-focus-ring-halo` | The tint, published beside it as the ring's halo. Unset with it. |

## Data attributes

State tokens on the `<blockquote>`'s `data-state`; the `<figcaption>` carries the
`size` token too. The `states` prop renders on the `<figure>` instead.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
