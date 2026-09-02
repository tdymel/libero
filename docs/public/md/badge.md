# Badge

Crate: `libero`
Import: `use libero::components::Badge;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/badge.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A short status label - one uppercase pill, sized under a control, with no role and no interaction.

A short status label. Renders one `<span>` with no role and no ARIA - a badge is
visible text, read in document order, so its content already is its accessible
name. The theme owns the uppercase, the letter spacing and the weight that tell a
badge from a chip at a glance; a project flips them once in `BadgeDefaults`
rather than per call site.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Badge;

#[component]
fn Demo() -> Element {
    rsx! {
        Badge { "New" }
    }
}
```

```rust
use dioxus::prelude::*;
use libero::components::{Badge, Flex};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex { direction: "row", gap: "sm", align: "center",
            Badge { color: "success", "Shipped" }
            Badge { variant: "outlined", size: "sm", color: "warning", "Beta" }
            Badge { circle: true, color: "error", "9" }
        }
    }
}
```

`Badge` labels; [`Chip`](chip.md) is picked - if the thing can be selected,
clicked or followed it is a `Chip`, which carries `checked`, `onclick` and `to`.
A badge has none of that and never changes under the cursor.

There is no `left_section`/`right_section` - the root is a flex container with a
`gap` of one spacing step, so a `Flex` child, or two direct children, does the
same job with nothing new to learn.

```rust
use dioxus::prelude::*;
use libero::components::{Badge, Flex, Icon};

#[component]
fn Demo() -> Element {
    rsx! {
        Badge {
            color: "success",
            Flex { direction: "row", align: "center", gap: "xs",
                Icon { size: "xs",
                    svg { view_box: "0 0 24 24", fill: "currentColor",
                        path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
                    }
                }
                "Verified"
            }
        }
    }
}
```

## Accessibility

- **No role, no ARIA.** The label is visible text and is its own accessible
  name. A `role="status"` would make every badge announce itself on mount; when
  a badge really does report a change, put the caller's own `role="status"`
  region around it and spread it through `attributes`.
- **Not focusable and not interactive**, so there is no keyboard contract. A
  caller who spreads `tabindex: "0"` gets the library's shared focus ring.
- **Contrast is the palette's, not the component's.** `filled` pairs the
  accent with its own `-contrast` twin, and `tonal` with the twin of its tint.
  At shade 6 some roles do not clear 4.5:1 for text this size - measured in
  Chromium at `md` (11px/700): `filled` is 3.56:1 primary, 3.28:1 error,
  2.79:1 info, against 4.95:1 secondary, 8.90:1 success, 11.28:1 warning,
  11.46:1 neutral; `tonal` is 15.9-18.4:1 for every role; `elevated`,
  `outlined` and `standard` put the accent on the page, which is 1.86:1 for
  warning up to 11.46:1 for neutral. `Button` and `Chip` sit on the same
  numbers. Pick `tonal`, or `neutral`, where the label has to be read rather
  than noticed.
- **The label is clipped, not ellipsised**, when it is wider than what holds
  it: one element has no inner span for `text-overflow` to act on. Badges are
  for short labels.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `ButtonVariant` | `filled` | Chrome, shared with `Button` and `Chip`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. A badge is not interactive, so it takes no hover response. |
| `color` | `ThemeAwareValue` | `primary` | The accent; a theme color name or a literal CSS color. A theme color also brings the `-contrast` twin the label reads with. |
| `size` | `Size` | `md` | Height, horizontal padding and font size, on the badge's own scale - smaller than a chip's. |
| `radius` | `ThemeAwareValue` | `9999px` | A size step or any CSS length. The theme's own default is off the scale, because a badge is a pill at every height. |
| `circle` | `bool` | `false` | Drops the horizontal padding and floors the width at the height, for a one- or two-character count. |
| `children` | `Element` | required | The label. |

Like every component, `Badge` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`BadgeDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Size step used when a call site names none - `md`. |
| `radius` | `&'static str` | Corner radius as a CSS length - `9999px`, off the radius scale, because a badge is a pill at every height. |
| `text_transform` | `&'static str` | `uppercase`. Most of what tells a badge from a chip at a glance; set it to `none` for a badge that carries a name. |
| `letter_spacing` | `&'static str` | `0.25px`, which opens up the uppercase. |
| `font_weight` | `&'static str` | `700`. |
| `sizes` | `Sizes<BadgeSizeLevel>` | `font_size`/`height`/`padding_x` per step: `0.5625rem/16px/6px`, `0.625rem/18px/8px`, `0.6875rem/20px/10px`, `0.8125rem/26px/12px`, `1rem/32px/16px`, `1.125rem/38px/20px`. |

`variant` and `color` are not theme fields - `ButtonVariant` is component-layer,
and `color` falls back to `primary` shade 6 in the component.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-badge-font-size-<size>` | Font size for that step, from `BadgeDefaults`. |
| `--lsx-badge-height-<size>` | Height for that step. |
| `--lsx-badge-padding-x-<size>` | Horizontal padding for that step. |
| `--lsx-badge-font` | The active step's font size, republished unsuffixed. |
| `--lsx-badge-box` | The active step's height. `circle` reads it for its `min-width`, and the line height is `calc()`ed from it. |
| `--lsx-badge-pad-x` | The active step's horizontal padding. |
| `--lsx-badge-radius` | The theme's corner radius. |
| `--lsx-badge-radius-override` | Set by the `radius` prop; wins over the theme's. |
| `--lsx-badge-text-transform` | From `BadgeDefaults::text_transform`. |
| `--lsx-badge-letter-spacing` | From `BadgeDefaults::letter_spacing`. |
| `--lsx-badge-font-weight` | From `BadgeDefaults::font_weight`. |
| `--lsx-badge-color` | Resolved `color`. |
| `--lsx-badge-contrast` | Text color on that accent, for `filled`. Unset for a literal CSS color. |
| `--lsx-badge-container` | Container fill of `tonal`. |
| `--lsx-badge-on-container` | Label color on that container - black or white, whichever reads on it. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
| `size-<size>` | The `size` in effect. |
| `circle` | `circle` is on. |
