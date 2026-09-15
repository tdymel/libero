# Blockquote

Crate: `libero`
Import: `use libero::components::Blockquote;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/typography/blockquote.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A quotation in a tinted frame with an accent bar, rendered as `figure` + `blockquote` + `figcaption` so the attribution sits outside the quote.

A quotation with its attribution. Renders a real `<figure>` holding a
`<blockquote>` and, when there is one, a `<figcaption>` - the attribution sits
outside the quote, which is where the HTML spec puts it, so assistive technology
does not read a speaker's name as quoted words. `size` scales the body text along
with the frame, on the same scale [Text](text.md) uses. The accent bar is on the
left in every writing direction.

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

Every prop set:

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

A bare theme color name (`"info"`) gives a shade-1 tint, a shade-6 accent bar and
the tint's own contrast color for the text. An explicit shade (`"info.2"`) tints
at that shade, keeps the shade-6 bar and takes that shade's contrast color. The
tint is the shade's fill color, the one its contrast color is computed on. A
literal CSS color (`"gold"`) is used as-is for both the background and the bar.
A hex sets black or white text; any other literal leaves the text to inherit,
because no contrast color can be derived from it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Body font size, line height, padding and the accent bar's width. |
| `color` | `ThemeAwareValue` | `primary`, tinted | The accent bar, and the background tint derived from it. A bare theme color is tinted to its lightest shade. A theme color paints its fill shade under its contrast twin as text. |
| `radius` | `Size` | `sm` | Rounds the two corners away from the accent bar. |
| `attribution` | `Element` | - | Who said it, rendered in a `<figcaption>` outside the quote. A person's name goes here as plain text - it does not become a `<cite>`. |
| `work` | `String` | - | The title of the work quoted, rendered as a `<cite>` in the `<figcaption>`. Not a person. Follows `attribution` after a comma when both are set. |
| `cite_url` | `String` | - | The `cite` attribute on `<blockquote>`: a URL naming the source document. Machine-readable only, no browser renders it. |
| `children` | `Element` | required | The quote. |

Like every component, `Blockquote` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the `<figure>`.

`attribution`, `work` and `cite_url` are three different things: a person's name
as plain text, the title of a work as a `<cite>` element, and the `cite`
attribute on the `<blockquote>`. They are independent. For any join other than
the comma, pass the whole line as `attribution` - it is an `Element`, so it can
hold its own `<cite>`.

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

The font size and line height are not `Blockquote`'s own: they come from
`TextDefaults`' scale, so retuning `theme.text` retunes quotes with the prose
around them. The `<figcaption>` is 0.85 of the same step's font size - spelled
out per size rather than as an `em`, because it is a sibling of the quote and an
`em` there would resolve against the `<figure>`.

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
