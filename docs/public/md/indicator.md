# Indicator

Crate: `libero`
Import: `use libero::components::Indicator;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/indicator.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A dot or a small capped count pinned to something else with a `Float` - presentational only, and never announced itself.

A dot or a small count pinned to something else - an unread marker on an avatar,
a pending count on a button. Renders one `<span aria-hidden="true">` and nothing
that positions it: the corner, the offset and the layer belong to
[`Float`](float.md), inside a `position: relative` parent. To hide it, do not
render it - there is no `disabled` or `show_zero`.

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

`label: 128` renders `99+`: the theme caps counts at 99, and `max` moves the cap
per call site. The cap is inclusive - `99` renders `99`.

Every positional decision above is a `Float` prop. `Float`'s `top-end` puts the
indicator inside the corner of the parent's box; on a round avatar that corner
lies outside the circle, so the dot sits on the edge without an offset. Use
`offset_x`/`offset_y` (a spacing token, or `"-xs"` for the other direction) to
move it.

```rust
use dioxus::prelude::*;
use libero::components::Indicator;

#[component]
fn Demo(unread: u32) -> Element {
    rsx! {
        // Hiding it is ordinary dioxus, not a prop.
        if unread > 0 {
            Indicator { label: unread, max: 9, color: "primary", processing: true }
        }
    }
}
```

## Accessibility

- **`aria-hidden="true"` by default, dot or count.** A bare dot is an empty
  node. A count is a truncation (`99+` where the truth is 128) and names
  nothing, so read on its own it says "ninety-nine plus" and not what is
  counted.
- **The meaning belongs on what the indicator marks.** The avatar or button
  carries the count in its own name - `alt: "Ada Lovelace, 128 unread"`,
  `aria_label: "Messages, 128 unread"`. The noun is the caller's vocabulary.
- `aria-hidden` goes on through `attr_default`, so a caller who wants the
  indicator itself read out can pass `aria_hidden: "false"` plus a
  `role="status"` of their own.
- **Not focusable and not interactive**, so there is no keyboard contract.
- `processing` stops under `prefers-reduced-motion: reduce`: the ping's
  animation is cancelled and the copy behind the dot stays hidden under it.
- **Contrast is the palette's.** The label is the fill's `-contrast` twin; at
  shade 6 the default `error` fill measures 3.28:1 with white, as `Badge`'s
  filled arm does. The count is decorative - its meaning is on the target - but
  pick a darker `color` shade where it has to be read.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Option<u32>` | `None` | The count. `None` is the bare dot. A number, not text, so `max` can cap it. |
| `max` | `Option<u32>` | `99` | Above it, the label renders as `{max}+`. The theme's cap unless set. |
| `size` | `Size` | `md` | The dot's diameter, and the height of a labelled one, on the indicator's own scale - 6px to 22px. |
| `color` | `ThemeAwareValue` | `error` | The fill; a theme color name or a literal CSS color. A theme color also brings the `-contrast` twin the label reads with. |
| `radius` | `ThemeAwareValue` | `9999px` | A size step or any CSS length. The theme's own default is round. |
| `with_border` | `bool` | `false` | A ring in the surface color, `--lsx-paper-background`, so the dot reads on top of a picture. |
| `processing` | `bool` | `false` | A ping growing and fading behind the dot. Stops under `prefers-reduced-motion`. |

Like every component, `Indicator` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`IndicatorDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Size step used when a call site names none - `md`. |
| `color` | `Color` | The fill when a call site names none - `Error`. |
| `radius` | `&'static str` | `9999px`, off the radius scale: a dot is round at every diameter. |
| `max` | `u32` | The count cap - `99`. A house convention, set once. |
| `border_width` | `&'static str` | The `with_border` ring - `2px`. |
| `processing_duration` | `&'static str` | One ping cycle - `1000ms`. |
| `sizes` | `Sizes<IndicatorSizeLevel>` | `size`/`font_size` per step: `6px/8px`, `8px/9px`, `10px/10px`, `14px/11px`, `18px/12px`, `22px/14px`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-indicator-size-<size>` | Diameter for that step, from `IndicatorDefaults`. |
| `--lsx-indicator-font-size-<size>` | Label font size for that step. |
| `--lsx-indicator-box` | The active step's diameter, republished unsuffixed. |
| `--lsx-indicator-font` | The active step's font size. |
| `--lsx-indicator-radius` | The theme's corner radius. |
| `--lsx-indicator-radius-override` | Set by the `radius` prop; wins over the theme's. |
| `--lsx-indicator-border-width` | The ring's width. Its color is `--lsx-paper-background`. |
| `--lsx-indicator-processing-duration` | One ping cycle. |
| `--lsx-indicator-color` | Resolved `color`. |
| `--lsx-indicator-contrast` | Label color on that fill. Unset for a literal CSS color. |

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
