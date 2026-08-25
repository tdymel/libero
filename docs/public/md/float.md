# Float

Crate: `libero`
Import: `use libero::components::Float;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/float.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Anchors its child to a corner or edge of the nearest positioned ancestor - a badge on an avatar, say.

Anchors its child to a corner/edge of the nearest `position: relative` ancestor -
e.g. a badge on an avatar. The parent must set `position: relative` itself.
`offset_x`/`offset_y` take a size token from the spacing scale, or any CSS length
- a negative one nudges the child back inward, which is how a badge overlaps its
anchor.

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

A negative offset pulls the child back over its anchor - the usual badge look:

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
                offset_x: "-8px",
                offset_y: "-8px",
                Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }
            }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `placement` | `Placement` | `center-center` | Anchor corner/edge, e.g. `"top-start"`. |
| `offset_x` | `ThemeAwareValue` | `0px` | Shift along the horizontal axis - a size token from the spacing scale, or any CSS length. Negative nudges the child inward. |
| `offset_y` | `ThemeAwareValue` | `0px` | Shift along the vertical axis. |
| `z_index` | `ThemeAwareValue` | `200` | Stacking order. |
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
