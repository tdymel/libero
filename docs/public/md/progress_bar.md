# ProgressBar

Crate: `libero`
Import: `use libero::components::ProgressBar;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/progress_bar.rs>
Index: [index.md](index.md) lists every other page
Description: A determinate or indeterminate progress bar over any `min..=max` range.

A bar that fills from `min` to `max`. It shows output and takes no focus. With
`value: None` it sweeps instead, for the time before the total is known.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::ProgressBar;

#[component]
fn Demo() -> Element {
    let uploaded = use_signal(|| 40.0);

    rsx! {
        ProgressBar {
            aria_label: "Upload",
            value: uploaded(),
            size: "xxl",
        }
    }
}
```

`value` is required. A bare number works, and `value: None` is the
indeterminate bar.

```rust,ignore
ProgressBar { aria_label: "Connecting", value: None }
```

## Accessibility

Name it with `aria_label`, or `aria_labelledby` pointing at a visible caption.
A screen reader reads the rounded percentage, or `aria_valuetext` when you set
it. The bar is not a live region. To announce progress, update a separate
status line at milestones, not on every tick.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<f64>` | required | Current progress, clamped into `min..=max`. `None` makes it indeterminate. |
| `min` | `f64` | `0.0` | Range start. |
| `max` | `f64` | `100.0` | Range end. At or below `min` the bar draws empty. |
| `color` | `ThemeAwareValue` | `primary` | The fill. A theme color name or any CSS color. |
| `size` | `Size` | `md` | Track height, 3px at `xs` to 20px at `xxl`. |
| `radius` | `Size` | `xl` | Corner of the track and the fill. On a thin track most steps draw the same pill. |
| `aria_valuetext` | `String` | - | Read instead of the rounded percentage, such as "4.2 MB of 12 MB". |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes, `aria_label` among them.

## Theme defaults

`ProgressBarDefaults` on the theme, as `theme.progress_bar`.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted (`md`). |
| `radius` | `Size` | Default `radius` when the prop is omitted (`xl`). |
| `track_shade` | `ColorShade` | Grey step of the unfilled track (`S2`). |
| `transition` | `&'static str` | How long the fill eases to a new value (`100ms`). |
| `sizes` | `Sizes<&'static str>` | Track height per size step: `3px 5px 8px 12px 16px 20px`. |

Where the radius stops mattering, radius scale `2/4/8/16/32/64px`:

| Track | Height | Pill from | Radius steps that look different |
|---|---|---|---|
| `xs` | 3px | `xs` | none |
| `sm` | 5px | `sm` | `xs` |
| `md` | 8px | `sm` | `xs` |
| `lg` | 12px | `md` | `xs`, `sm` |
| `xl` | 16px | `md` | `xs`, `sm` |
| `xxl` | 20px | `lg` | `xs`, `sm`, `md` |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-progress-bar-thickness-<size>` | Track height for that size step. |
| `--lsx-progress-bar-size` | The picked height, resolved on the root. |
| `--lsx-progress-bar-radius` | The picked radius, resolved on the root; the fill inherits the corner. |
| `--lsx-progress-bar-track` | The unfilled track, `muted.2`. Themed once. |
| `--lsx-progress-bar-transition` | Duration of the fill's `width` transition. Themed once. |
| `--lsx-progress-bar-color` | The resolved `color`, per instance. |
| `--lsx-progress-bar-fill` | The drawn percentage, e.g. `42%`, per instance. Not written while indeterminate, and nothing reads it then. |

The sweep is `@keyframes lsx-progress-bar-indeterminate`, emitted once in the
theme stylesheet.

## Data attributes

`data-state` on the root and on the fill carries `size-*`, `radius-*`, and
either `determinate` or `indeterminate`.
