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
it. The child is stretched to fill the badge (`& svg { width: 100%; height: 100% }`).

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
            Icon { variant: "transparent", color: "info", size: "lg",
                svg { view_box: "0 0 24 24", fill: "currentColor",
                    path { d: "M9 16.2 4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4z" }
                }
            }
        }
    }
}
```

## Accessibility

`Icon` is decorative chrome and adds no semantics of its own - it is a `<span>`
(or whatever `component` says) with no role and no accessible name. Give the svg
a `<title>`, or the wrapper an `aria_label`, when the icon carries meaning on its
own; mark it `aria_hidden: "true"` when it merely repeats adjacent text. For a
clickable icon use [`ActionIcon`](action_icon.md), which is a real `<button>`
with a required `aria_label`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `component` | `HtmlTag` | `span` | Element to render as. |
| `variant` | `ButtonVariant` | `filled` | Chrome around the svg, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. A badge is not interactive, so it takes no hover response. |
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
| `size` | `Sizes<u16>` | Badge width/height in px per size step - `16, 20, 24, 32, 40, 48` by default. |

The `variant`, `color`, `size` and `radius` fallbacks are the component's own
(`filled`, `primary`, `md`, `sm`), not theme fields.

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
