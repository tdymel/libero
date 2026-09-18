# Icon

Crate: `libero`
Import: `use libero::components::Icon;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/icon.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A sized, colored badge around an svg child, whose `currentColor` fill inherits the badge's color.

Wraps an svg child in a sized, colored badge. `color` sets the container's CSS
`color`, which any child svg using `currentColor` for its fill/stroke then
inherits. It renders a `<span>` by default and shrink-wraps to the size step it
is given, so it never stretches inside a flex row.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Icon;

#[component]
fn Demo() -> Element {
    rsx! {
        Icon {
            variant: "filled",
            svg {
                view_box: "0 0 24 24",
                fill: "currentColor",
                path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
            }
        }
    }
}
```

Any svg works, including one from an icon crate - `Icon` only sizes and colors
it. With `standard`, which draws no container, the svg fills the box. Every other
variant insets it to 60% of the box, so the glyph keeps clear of the container's
edges. Set `--lsx-icon-glyph` in `sx` to pick another share.

```rust
use dioxus::prelude::*;
use libero::components::{Flex, Icon};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md",
            Icon { variant: "filled", color: "success", size: "lg", radius: "xl",
                svg { view_box: "0 0 24 24", fill: "currentColor",
                    path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
                }
            }
            Icon { variant: "outlined", color: "error", size: "lg", radius: "xl",
                svg { view_box: "0 0 24 24", fill: "currentColor",
                    path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
                }
            }
            Icon { variant: "standard", color: "info", size: "lg",
                svg { view_box: "0 0 24 24", fill: "currentColor",
                    path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
                }
            }
        }
    }
}
```

## Accessibility

`Icon` is decorative by default: `aria-hidden="true"`, so it adds nothing next
to the text it repeats. When the icon carries meaning on its own, give it an
`aria_label` (or `aria_labelledby`): it becomes `role="img"` under that name.

**A `<title>` inside the svg does not name the icon.** It is hidden together
with the rest of the unnamed `Icon`; move its text into `aria_label`.

For a clickable icon use [`ActionIcon`](action_icon.md), which requires an
`aria_label`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `component` | `HtmlTag` | `span` | Element to render as. |
| `variant` | `Variant` | `filled` | Chrome around the svg, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. A badge is not interactive, so it takes no hover response. |
| `color` | `ThemeAwareValue` | `primary` | Sets the container's CSS color, which a `currentColor` svg then inherits. A theme color also tints the background under variant `filled`. |
| `size` | `ThemeAwareValue` | `md` | Badge width and height. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius of the badge. |
| `children` | `Element` | required | The svg to badge. |

Like every component, `Icon` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`IconDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`filled`). |
| `sizes` | `Sizes<u16>` | Badge width/height in px per size step - `16, 20, 24, 32, 40, 48` by default. |

The `color`, `size` and `radius` fallbacks are the component's own
(`primary`, `md`, `sm`), not theme fields.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-icon-size-<size>` | Width/height for that size step, from `IconDefaults`. |
| `--lsx-icon-size-override` | Set by the `size` prop; wins over the size step. |
| `--lsx-icon-color` | Resolved `color`; the svg inherits it as `currentColor`. |
| `--lsx-icon-contrast` | Text color on top of that accent, for variant `filled`. Unset for a literal CSS color. |
| `--lsx-icon-radius` | Set by the `radius` prop; falls back to `--lsx-radius-sm`. |
| `--lsx-icon-container` | Container fill of `tonal`. |
| `--lsx-icon-on-container` | Label color on that container - black or white, whichever reads on it. |

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `filled` / `tonal` / `elevated` / `outlined` / `standard` | The `variant` in effect. |
