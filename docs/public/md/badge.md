# Badge

Crate: `libero`
Import: `use libero::components::Badge;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/badge.rs>
Index: [index.md](index.md) lists every other page
Description: A short status label, one uppercase pill with no role and no interaction.

A short status label, rendered as one `<span>`. An icon and text as children
sit one spacing step apart. The theme sets the uppercase, letter spacing and
weight in `BadgeDefaults`. For something a user can select, click or follow,
use [`Chip`](chip.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Badge, Icon};

#[component]
fn Demo() -> Element {
    rsx! {
        Badge { "New" }
        Badge { circle: true, color: "error", "9" }
        Badge { color: "success",
            Icon { size: "xs",
                svg { view_box: "0 0 24 24", fill: "currentColor",
                    path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
                }
            }
            "Verified"
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | `filled` | The look, shared with `Button` and `Chip`. A badge is not interactive, so it has no hover state. |
| `color` | `ThemeAwareValue` | `primary` | A theme color name or a CSS color. A theme color also sets a label color that reads on it. Under a gradient, its first stop. |
| `gradient` | `Gradient` | - | With `variant: "gradient"`: the second stop and the angle, as `("info", 90)` or `Gradient::default().to("info").deg(90)`. The first stop is `color`. Ignored by the other variants. |
| `size` | `Size` | `md` | Height, horizontal padding and font size, on a scale smaller than a chip's. |
| `radius` | `Size` | `xxl` | A step on the badge's own radius scale, `2px` to `12px`. The default `xxl` is a pill at every height. |
| `circle` | `bool` | `false` | Drops the horizontal padding and makes the width at least the height, for a count of one or two characters. |
| `children` | `Element` | required | The label. |

Like every component, `Badge` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- A badge has no role, so screen readers read its text in place and announce no
  change.
- `filled` and `tonal` labels reach 4.5:1 in every color.

### You must

- For a badge that reports a change, wrap it in your own `role="status"`
  region.
- Pick `filled` or `tonal` for `warning` and `success`.

### Example

A "Paid" badge in an invoice row, `Badge { color: "success", variant: "tonal",
"Paid" }`: read in place with the row, at 4.5:1 or more.

### Limits

- The other variants print the label in the color itself, which stays under
  4.5:1 on white for `warning` (3.27:1) and `success` (4.05:1).

## Theme defaults

`BadgeDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Variant when the prop is omitted, `filled`. |
| `size` | `Size` | Size step when the prop is omitted, `md`. |
| `radius` | `Size` | Step of `radii` when the prop is omitted, `xxl`, a pill at every height. |
| `text_transform` | `&'static str` | `uppercase`. Set it to `none` for a badge that carries a name. |
| `letter_spacing` | `&'static str` | `0.25px`, which opens up the uppercase. |
| `font_weight` | `&'static str` | `700`. |
| `sizes` | `Sizes<BadgeSizeLevel>` | `font_size`/`height`/`padding_x` per step: `0.5625rem/1rem/0.375rem`, `0.625rem/1.125rem/0.5rem`, `0.6875rem/1.25rem/0.625rem`, `0.8125rem/1.625rem/0.75rem`, `1rem/2rem/1rem`, `1.125rem/2.375rem/1.25rem`. All rem, so the box grows with the reader's text size. |
| `radii` | `Sizes<&'static str>` | The badge's own radius scale: `2px`, `4px`, `6px`, `8px`, `12px`, `9999px`. |

`color` is not a theme field. It falls back to `primary` shade 6.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-badge-font-size-<size>` | Font size for that step, from `BadgeDefaults`. |
| `--lsx-badge-height-<size>` | Height for that step. |
| `--lsx-badge-padding-x-<size>` | Horizontal padding for that step. |
| `--lsx-badge-font` | The active step's font size, republished unsuffixed. |
| `--lsx-badge-box` | The active step's height. `circle` reads it for its `min-width`, and the line height is `calc()`ed from it. |
| `--lsx-badge-pad-x` | The active step's horizontal padding. |
| `--lsx-badge-radius-<size>` | The radius for that step, from `BadgeDefaults::radii`. |
| `--lsx-badge-radius` | The theme's default step, as `var(--lsx-badge-radius-xxl)`. |
| `--lsx-badge-radius-override` | Set by the `radius` prop to that step's var; wins over the theme's. |
| `--lsx-badge-text-transform` | From `BadgeDefaults::text_transform`. |
| `--lsx-badge-letter-spacing` | From `BadgeDefaults::letter_spacing`. |
| `--lsx-badge-font-weight` | From `BadgeDefaults::font_weight`. |
| `--lsx-badge-color` | Resolved `color`. |
| `--lsx-badge-contrast` | Text color on that accent, for `filled`. For a literal CSS color, black or white, whichever reads on it. |
| `--lsx-badge-container` | Container fill of `tonal`. |
| `--lsx-badge-on-container` | Label color on that container, black or white, whichever reads on it. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `size-<size>` | The `size` in effect. |
| `circle` | `circle` is on. |
