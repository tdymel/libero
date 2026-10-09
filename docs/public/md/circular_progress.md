# CircularProgress

Crate: `libero`
Import: `use libero::components::CircularProgress;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/circular_progress.rs>
Index: [index.md](index.md) lists every other page
Description: A ring that fills clockwise from the top over any `min..=max` range, or spins, with an optional label in its middle.

A ring that fills clockwise from the top, for a percentage in a small space: on
a button, around an avatar, beside a step count. It shows output and takes no
focus. With `value: None` it spins instead, for the time before the total is
known.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::CircularProgress;

#[component]
fn Demo() -> Element {
    let uploaded = use_signal(|| 40.0);

    rsx! {
        CircularProgress {
            aria_label: "Upload",
            value: uploaded(),
            size: "xl",
            "{uploaded}%"
        }
    }
}
```

`value` is required. A bare number works, and `value: None` is the spinning
ring.

```rust,ignore
CircularProgress { aria_label: "Connecting", value: None }
```

A step count reads better than a percentage: say it in `aria_valuetext`.

```rust
use dioxus::prelude::*;
use libero::components::CircularProgress;

#[component]
fn Demo() -> Element {
    let sent = use_signal(|| 3.0);

    rsx! {
        CircularProgress {
            aria_label: "Files sent",
            value: sent(),
            max: 8.0,
            size: "xl",
            aria_valuetext: format!("{sent} of 8 files"),
            "{sent}/8"
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<f64>` | required | Current progress, clamped into `min..=max`. `None` makes it spin. |
| `min` | `f64` | `0.0` | Range start. |
| `max` | `f64` | `100.0` | Range end. At or below `min` the ring draws empty. |
| `color` | `ThemeAwareValue` | `primary` | The arc. A theme color name paints its text shade, as `ProgressBar`'s fill. Any other CSS color paints as given. |
| `size` | `Size` | `md` | The outer edge, 18px at `xs` to 72px at `xxl` at the default text size; it grows with text zoom. |
| `thickness` | `Size` | `md` | The ring's width as a share of the edge, 6% at `xs` to 16% at `xxl`, so it grows with `size`. |
| `aria_valuetext` | `String` | - | Read instead of the rounded percentage, such as "3 of 8 files". |
| `children` | `Element` | - | Drawn in the middle of the ring, such as the percentage or an icon, at a quarter of the edge, never under 12px. Hidden from screen readers: say the same in `aria_valuetext` when it differs from the percentage. |
| `parts` | `Parts<CircularProgressPart>` | - | Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(CircularProgressPart::Label, sx().font_weight("bold"))`. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes, `aria_label` among them.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `CircularProgressPart::Arc` | `arc` | The `<svg>` holding the value arc. The root is the track. |
| `CircularProgressPart::Label` | `label` | The centred children. |

## Accessibility

### Libero handles

- A screen reader reads the rounded percentage, or `aria_valuetext` when you
  set it. A spinning ring reports no value.
- The ring takes no focus. The centred children are hidden from screen readers,
  so the value is not read twice.
- A theme color draws the arc in its text shade, at 3:1 or more against the
  track and the page. Yellow stays short of that on a light page, so `warning`
  draws its arc on a wider ink ring.
- With reduced motion a spinning ring stops and shows as a full dashed ring, so
  it does not read as a quarter done.

### You must

- Name it with `aria_label`, or `aria_labelledby` pointing at a visible
  caption.
- The ring is not a live region. To announce progress, update a separate status
  line at milestones, not on every tick.
- Pick a `size` whose middle fits the children: the text is never under
  12px, so it spills over an `xs` or `sm` ring, and `md` holds about three
  characters ("42%", not "100%").

### Example

An upload ring, `CircularProgress { value: Some(42.0), aria_label: "Upload", "42%" }`:
a screen reader reads the name "Upload" and 42% when it reaches the ring.

## Theme defaults

`CircularProgressDefaults` on the theme, as `theme.circular_progress`.

| Field | Type | Description |
|---|---|---|
| `color` | `Color` | Default `color` when the prop is omitted (`primary`). |
| `size` | `Size` | Default `size` when the prop is omitted (`md`). |
| `thickness` | `Size` | Default `thickness` when the prop is omitted (`md`). |
| `track_shade` | `ColorShade` | Grey step of the unfilled ring (`S2`). |
| `transition` | `&'static str` | How long the arc eases to a new value (`100ms`). |
| `sizes` | `Sizes<&'static str>` | Edge per size step, in rem: `1.125rem 1.375rem 2.25rem 2.75rem 3.625rem 4.5rem` (18px to 72px at 16px). |
| `thicknesses` | `Sizes<u8>` | Ring width per thickness step, in percent of the edge: `6 8 10 12 14 16`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-circular-progress-size-<size>` | Edge for that size step. |
| `--lsx-circular-progress-size` | The picked edge, resolved on the root. |
| `--lsx-circular-progress-track` | The unfilled ring, `muted.2`. Themed once. |
| `--lsx-circular-progress-transition` | Duration of the arc's ease. Themed once. |
| `--lsx-circular-progress-color` | The resolved `color` in its text shade, per instance. |
| `--lsx-circular-progress-thickness` | The ring's width as a share of the edge, per instance. |

The arc's length and offset are SVG attributes, not variables: a native window
draws an inline svg from its attributes. The spin reuses `Loader`'s
`@keyframes lsx-loader-oval`.

## Data attributes

`data-state` on the root carries `size-*` and either `determinate` or
`indeterminate`. `data-slot="arc"` and `data-slot="label"` mark the parts, see
[Style API](#style-api).
