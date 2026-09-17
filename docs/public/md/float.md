# Float

Crate: `libero`
Import: `use libero::components::Float;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/float.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Anchors its child to a corner or edge of the nearest positioned ancestor - a badge on an avatar, say.

Anchors its child to a corner/edge of the nearest `position: relative` ancestor -
e.g. a badge on an avatar. The parent must set `position: relative` itself.
`offset_x`/`offset_y` take a size token from the spacing scale, or any CSS
length, and shift it right/down along the page axes - negate the token (`"-md"`)
to shift left/up instead.

## Usage

A float positions against the nearest `position: relative` ancestor, so the
parent in the snippet is part of the usage, not scaffolding.

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Float}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().position("relative").width("160px").height("120px").background("primary.1"),
            Float {
                placement: "top-end",
                Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }
            }
        }
    }
}
```

`placement` names the vertical half first: `top-start`, `top-center`, `top-end`,
`center-start`, `center-center`, `center-end`, `bottom-start`, `bottom-center`,
`bottom-end`. The centered axes are centered with a `translate(-50%)`, so an
offset is added on top of it rather than replacing it.

Offsets are page axes, not placement-relative: `"md"` shifts right/down, `"-md"`
left/up. Which of the two leaves the anchor therefore depends on the placement -
on a `top-end` badge it is a positive `offset_x` and a negative `offset_y` that
hang it off the corner:

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Float}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().position("relative").width("160px").height("120px").background("primary.1"),
            Float {
                placement: "top-end",
                offset_x: "sm",
                offset_y: "-sm",
                Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }
            }
        }
    }
}
```

`fixed: true` swaps `position: absolute` for `position: fixed`, so the float stays
put while the page scrolls - an action bar, a notification stack, a floating
window. Everything else is unchanged: the same `placement`, the same offsets, the
same `float` z-index layer, which sits below overlays and modals.

A `transform`, `filter`, `contain` or `container-type` on an ancestor makes that
ancestor the containing block of a fixed element, and it then scrolls and clips
with it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `placement` | `Placement` | `center-center` | Anchor corner/edge, e.g. `"top-start"`. |
| `offset_x` | `ThemeAwareValue` | `0px` | Shift along the horizontal axis - a size token from the spacing scale (`"md"`, or `"-md"` for the other direction), or any CSS length. |
| `offset_y` | `ThemeAwareValue` | `0px` | Shift along the vertical axis. |
| `z_index` | `ThemeAwareValue` | `200` | Stacking order. |
| `fixed` | `bool` | `false` | Place against the viewport (`position: fixed`) instead of the nearest positioned ancestor. `placement` and the offsets are measured from the viewport's edges. |
| `children` | `Element` | required | The anchored content. |

Like every component, `Float` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`FloatDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `offset_x` | `&'static str` | Horizontal offset when the prop is omitted. |
| `offset_y` | `&'static str` | Vertical offset when the prop is omitted. |
| `placement` | `Placement` | Default `placement` when the prop is omitted. |

`z_index` falls back to the theme's `z_index.float`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-float-offset-x` | Horizontal offset; the theme default, or the resolved `offset_x` prop. |
| `--lsx-float-offset-y` | Vertical offset; the theme default, or the resolved `offset_y` prop. |
| `--lsx-float-translate-x` | Set to `-50%` on a horizontally centered placement; the offset is added to it. |
| `--lsx-float-translate-y` | Set to `-50%` on a vertically centered placement. |
| `--lsx-z-index-float` | Theme stacking order for floats. |
| `--lsx-z-index-float-override` | Set from the `z_index` prop; wins over the theme value. |

## Data attributes

State tokens on the root's `data-state`, space separated - one per axis, derived
from `placement`.

| Token | Condition |
|---|---|
| `vertical-top` / `vertical-center` / `vertical-bottom` | The vertical half of `placement`. |
| `horizontal-start` / `horizontal-center` / `horizontal-end` | The horizontal half of `placement`. |
