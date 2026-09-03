# ProgressBar

Crate: `libero`
Import: `use libero::components::ProgressBar;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/feedback/progress_bar.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A determinate or indeterminate progress bar over any `min..=max` range, with `role="progressbar"` and the raw `aria-value*` set on its root.

A bar that fills from `min` to `max`. It is output, not a control: it takes no
focus and no keys. The root carries `role="progressbar"` and the raw
`aria-valuenow`/`min`/`max`, so it needs a name - pass `aria_label`, which lands
on the root. The fill eases to each new value; with `value: None` it sweeps
instead, for the time before the total is known.

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

`value` is required and is an `Option<f64>`: a bare number works through
`into`, and `value: None` is the indeterminate bar.

```rust
ProgressBar { aria_label: "Connecting", value: None }
```

## Announcing progress

The bar is not a live region. One that ticks sixty times a second inside
`role="status"` floods the screen reader's queue until it is announcing numbers
from a minute ago. A reader who wants to know checks the bar. If progress has to
be announced, put a separate status line next to it and update it at milestones
- a quarter done, half done, finished - not on every tick.

## Accessibility

- One `div` with `role="progressbar"`; the fill inside it is `aria-hidden`. The
  role sits on the root, which always has size, rather than on the fill, which
  is zero wide at zero.
- `aria-valuemin`/`aria-valuemax` are the raw bounds. `aria-valuenow` is the
  value clamped into them, exactly as the fill is drawn.
- `aria-valuetext` defaults to the rounded percentage (`"42%"`). The prop
  replaces it; so does an `aria-valuetext` attribute spread by the caller, since
  the percentage is only a default.
- Indeterminate (`value: None`) omits `aria-valuenow` and `aria-valuetext`
  entirely, which is how ARIA says "busy, amount unknown".
- No accessible name is invented. Pass `aria_label`, or `aria_labelledby`
  pointing at a visible caption.
- Not focusable, no keyboard interaction.
- Reduced motion: the fill's transition is dropped, and the indeterminate sweep
  becomes a dimmed full-width fill that stands still, so it cannot be misread
  as a 25% bar.

## Props

### `ProgressBar`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<f64>` | required | Current progress, clamped into `min..=max`. `None` is indeterminate: the bar sweeps and `aria-valuenow` is dropped. |
| `min` | `f64` | `0.0` | Range start. |
| `max` | `f64` | `100.0` | Range end. At or below `min` it warns and draws empty. |
| `color` | `ThemeAwareValue` | `primary` | The fill; a theme color name or a literal CSS color. |
| `size` | `Size` | `md` | Track height, 3px at `xs` to 20px at `xxl`. |
| `radius` | `Size` | `xl` | Corner of the track and the fill. A track is a full pill once the radius reaches half its height, so on the default `md` track only `xs` looks different. |
| `aria_valuetext` | `String` | - | What a screen reader announces instead of the rounded percentage. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `aria_label` among them.

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
| `--lsx-progress-bar-track` | The unfilled track, `grey.2`. Themed once. |
| `--lsx-progress-bar-transition` | Duration of the fill's `width` transition. Themed once. |
| `--lsx-progress-bar-color` | The resolved `color`, per instance. |
| `--lsx-progress-bar-fill` | The drawn percentage, e.g. `42%`, per instance. Not written while indeterminate, and nothing reads it then. |

The sweep is `@keyframes lsx-progress-bar-indeterminate`, emitted once in the
theme stylesheet.

## Data attributes

`data-state` on the root and on the fill carries `size-*`, `radius-*`, and
either `determinate` or `indeterminate`.
