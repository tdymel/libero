# Indicator

Crate: `libero`
Import: `use libero::components::Indicator;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/indicator.rs>
Index: [index.md](index.md) lists every other page
Description: A dot or a small capped count pinned to something else with a `Float`, never read out itself.

A dot or a small count pinned to something else, such as an unread marker on an
avatar. It renders one `<span aria-hidden="true">` and does not position itself.
Put it in a [`Float`](float.md) inside a `position: relative` parent, which sets
the corner, the offset and the layer. To hide it, do not render it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::{Avatar, Box, Float, Indicator}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().position("relative").display("inline-flex"),
            Avatar { name: "Ada Lovelace", alt: "Ada Lovelace, 128 unread", size: "lg" }
            Float {
                placement: "top-end",
                Indicator { label: 128, with_border: true }
            }
        }
    }
}
```

`label: 128` renders `99+`. The theme caps counts at 99, and `max` moves the
cap. `99` itself renders `99`. On a round avatar the corner lies outside the
circle, so the dot sits on the edge. `Float`'s `offset_x` and `offset_y` move
it.

```rust
use dioxus::prelude::*;
use libero::components::Indicator;

#[component]
fn Demo(unread: u32) -> Element {
    rsx! {
        if unread > 0 {
            Indicator { label: unread, max: 9, color: "primary", processing: true }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Option<u32>` | `None` | The count. `None` is the bare dot. A number, so `max` can cap it. |
| `max` | `Option<u32>` | `99` | Above it, the label renders as `{max}+`. Falls back to the theme's cap. |
| `size` | `Size` | `md` | The dot's diameter, and the height of a labelled one, from 6px to 22px. |
| `color` | `ThemeAwareValue` | `error` | The fill, a theme color name or a CSS color. A theme or hex color also sets a label color that reads on it; any other CSS color gets `contrast-color()`. |
| `radius` | `ThemeAwareValue` | `xxl` | A step on the indicator's own radius scale, `1px` to `6px`, or any CSS, e.g. `radius: "0"`. The default `xxl` is round at every size. |
| `with_border` | `bool` | `false` | A ring in the surface color, so the dot reads on top of a picture. |
| `processing` | `bool` | `false` | A ping behind the dot that repeats until you set it back to `false`. Stops under `prefers-reduced-motion`. |

Like every component, `Indicator` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- Screen readers never read the indicator.
- A theme or hex color labels the count at 4.5:1 or better.

### You must

- Put the count in the name of what it marks, as the demo avatar's
  `alt: "Ada Lovelace, 128 unread"`, or `aria_label: "Messages, 128 unread"` on
  a button.
- To have the indicator read, pass `aria_hidden: "false"` and wrap it in your
  own `role="status"` region.
- `processing` pings until you turn it off. Set it back to `false` when the
  work ends, since motion that never stops fails WCAG 2.2.2.
- Give the fill 3:1 against what is around it (WCAG 1.4.11). A CSS color other
  than hex labels the count with `contrast-color()`, which a browser without it
  ignores.

### Example

An unread count on an avatar, `Indicator { label: 128 }` with `Avatar { alt:
"Ada Lovelace, 128 unread" }`: a screen reader hears the count once, in the
avatar's name, and never reads the indicator.

## Theme defaults

`IndicatorDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Size step when the prop is omitted, `md`. |
| `color` | `Color` | Fill when the prop is omitted, `Error`. |
| `radius` | `Size` | Step of `radii` when the prop is omitted, `xxl`, round at every size. |
| `max` | `u32` | The count cap, `99`. |
| `border_width` | `&'static str` | Width of the `with_border` ring, `2px`. |
| `processing_duration` | `&'static str` | One ping cycle, `1000ms`. |
| `sizes` | `Sizes<IndicatorSizeLevel>` | `size`/`font_size` per step: `6px/8px`, `8px/9px`, `10px/10px`, `14px/11px`, `18px/12px`, `22px/14px`. |
| `radii` | `Sizes<&'static str>` | The indicator's own radius scale: `1px`, `2px`, `3px`, `4px`, `6px`, `9999px`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-indicator-size-<size>` | Diameter for that step, from `IndicatorDefaults`. |
| `--lsx-indicator-font-size-<size>` | Label font size for that step. |
| `--lsx-indicator-box` | The active step's diameter, republished unsuffixed. |
| `--lsx-indicator-font` | The active step's font size. |
| `--lsx-indicator-radius-<size>` | The radius for that step, from `IndicatorDefaults::radii`. |
| `--lsx-indicator-radius` | The theme's default step, as `var(--lsx-indicator-radius-xxl)`. |
| `--lsx-indicator-radius-override` | Set by the `radius` prop to that step's var; wins over the theme's. |
| `--lsx-indicator-border-width` | The ring's width. Its color is `--lsx-paper-background`. |
| `--lsx-indicator-processing-duration` | One ping cycle. |
| `--lsx-indicator-color` | Resolved `color`. |
| `--lsx-indicator-contrast` | Label color on that fill: black or white for a hex color, `contrast-color()` for any other CSS color. |

The ping is `@keyframes lsx-indicator-processing`, appended to the theme
stylesheet.

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `labelled` | `label` is set; adds the horizontal padding. |
| `with-border` | `with_border` is on. |
| `processing` | `processing` is on. |
