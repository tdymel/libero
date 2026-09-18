# Badge

Crate: `libero`
Import: `use libero::components::Badge;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/badge.rs>
Index: [index.md](index.md) lists every other page
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

- **A badge announces nothing on its own.** When one really does report a
  change, put your own `role="status"` region around it.
- **Contrast is the palette's.** `filled` labels every role at 4.5:1 or
  better, and `tonal` at 15.9:1 or better. `elevated`, `outlined` and
  `standard` print the label in the colour's text role, which stays under
  4.5:1 on a white page for `warning` (3.27:1) and `success` (4.05:1). Pick
  `filled` or `tonal` for those where the label has to be read.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | `filled` | Chrome, shared with `Button` and `Chip`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. A badge is not interactive, so it takes no hover response. |
| `color` | `ThemeAwareValue` | `primary` | The accent; a theme color name or a literal CSS color. A theme color also brings the `-contrast` twin the label reads with. |
| `size` | `Size` | `md` | Height, horizontal padding and font size, on the badge's own scale - smaller than a chip's. |
| `radius` | `Size` | `xxl` | A step on the badge's own radius scale, `2px` to `12px`. The default, `xxl`, is `9999px`: a pill at every height. |
| `circle` | `bool` | `false` | Drops the horizontal padding and floors the width at the height, for a one- or two-character count. |
| `children` | `Element` | required | The label. |

Like every component, `Badge` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`BadgeDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Variant used when a call site names none - `filled`. |
| `size` | `Size` | Size step used when a call site names none - `md`. |
| `radius` | `Size` | The step of `radii` used when a call site names none - `xxl`, a pill at every height. |
| `text_transform` | `&'static str` | `uppercase`. Most of what tells a badge from a chip at a glance; set it to `none` for a badge that carries a name. |
| `letter_spacing` | `&'static str` | `0.25px`, which opens up the uppercase. |
| `font_weight` | `&'static str` | `700`. |
| `sizes` | `Sizes<BadgeSizeLevel>` | `font_size`/`height`/`padding_x` per step: `0.5625rem/1rem/0.375rem`, `0.625rem/1.125rem/0.5rem`, `0.6875rem/1.25rem/0.625rem`, `0.8125rem/1.625rem/0.75rem`, `1rem/2rem/1rem`, `1.125rem/2.375rem/1.25rem`. All rem, so the box grows with the reader's text size. |
| `radii` | `Sizes<&'static str>` | The badge's own radius scale: `2px`, `4px`, `6px`, `8px`, `12px`, `9999px`. |

`color` is not a theme field - it falls back to `primary` shade 6 in the component.

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
