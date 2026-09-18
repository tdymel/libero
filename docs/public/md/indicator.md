# Indicator

Crate: `libero`
Import: `use libero::components::Indicator;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/indicator.rs>
Index: [index.md](index.md) lists every other page
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

- **The meaning belongs on what the indicator marks**, because the indicator is
  never read out. The avatar or button carries the count in its own name -
  `alt: "Ada Lovelace, 128 unread"`, `aria_label: "Messages, 128 unread"`. The
  noun is the caller's vocabulary.
- To have the indicator itself read out, pass `aria_hidden: "false"` plus a
  `role="status"` of your own.
- **`processing` pings until you turn it off.** Set it back to `false` when the
  work ends: motion that starts on its own and never stops fails WCAG 2.2.2.
  Only `prefers-reduced-motion` stops it for you.
- **Contrast is the palette's.** A theme color's fill labels the count at
  4.5:1 or better. A literal CSS `color` brings no contrast twin, so check its
  label yourself.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Option<u32>` | `None` | The count. `None` is the bare dot. A number, not text, so `max` can cap it. |
| `max` | `Option<u32>` | `99` | Above it, the label renders as `{max}+`. The theme's cap unless set. |
| `size` | `Size` | `md` | The dot's diameter, and the height of a labelled one, on the indicator's own scale - 6px to 22px. |
| `color` | `ThemeAwareValue` | `error` | The fill; a theme color name or a literal CSS color. A theme color also brings the `-contrast` twin the label reads with. |
| `radius` | `Size` | `xxl` | A step on the indicator's own radius scale, `1px` to `6px`. The default, `xxl`, is `9999px`: round at every diameter. |
| `with_border` | `bool` | `false` | A ring in the surface color, `--lsx-paper-background`, so the dot reads on top of a picture. |
| `processing` | `bool` | `false` | A ping growing and fading behind the dot, repeating until you set it back to `false`. Stops under `prefers-reduced-motion`. |

Like every component, `Indicator` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`IndicatorDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Size step used when a call site names none - `md`. |
| `color` | `Color` | The fill when a call site names none - `Error`. |
| `radius` | `Size` | The step of `radii` used when a call site names none - `xxl`, round at every diameter. |
| `max` | `u32` | The count cap - `99`. A house convention, set once. |
| `border_width` | `&'static str` | The `with_border` ring - `2px`. |
| `processing_duration` | `&'static str` | One ping cycle - `1000ms`. |
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
