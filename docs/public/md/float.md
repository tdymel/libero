# Float

Crate: `libero`
Import: `use libero::components::Float;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/float.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Anchors its child to a corner or edge of the nearest positioned ancestor, like a badge on an avatar.

Anchors its child to a corner or edge of the nearest positioned ancestor, like a
badge on an avatar. The parent sets `position: relative` itself. The offsets
follow the page, not the placement, so on a `top-end` badge a positive
`offset_x` and a negative `offset_y` hang it off the corner.

## Usage

The parent `Box` is the positioned ancestor.

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Float}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().position("relative").width("160px").height("120px").background("primary.1"),
            Float {
                Box { sx: sx().padding("4px 8px").background("primary"), "Badge" }
            }
        }
    }
}
```

`placement` names the vertical half first: `top-start`, `top-center`, `top-end`,
`center-start`, `center-center`, `center-end`, `bottom-start`, `bottom-center`,
`bottom-end`. An offset of `"md"` shifts right or down, `"-md"` left or up.

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

With `fixed: true` the float anchors to the viewport and stays put while the
page scrolls, for an action bar or a notification stack. `placement` and the
offsets work the same. It stays below overlays and modals.

An ancestor with a `transform`, `filter`, `contain` or `container-type` still
captures a fixed float, which then scrolls and clips with it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `placement` | `Placement` | `center-center` | The corner or edge to anchor to, such as `"top-start"`. |
| `offset_x` | `ThemeAwareValue` | `0px` | Shift to the right, a spacing step or a CSS length. A negative step like `"-md"` shifts left. |
| `offset_y` | `ThemeAwareValue` | `0px` | Shift down, a spacing step or a CSS length. A negative step shifts up. |
| `z_index` | `ThemeAwareValue` | `200` | Stacking order. |
| `fixed` | `bool` | `false` | Anchors to the viewport instead of the parent, so it stays put while the page scrolls. An ancestor with a `transform`, `filter`, `contain` or `container-type` still captures it. |
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
| `--lsx-float-offset-x` | Horizontal offset, from `offset_x` or the theme. |
| `--lsx-float-offset-y` | Vertical offset, from `offset_y` or the theme. |
| `--lsx-float-translate-x` | `-50%` on a horizontally centered placement. |
| `--lsx-float-translate-y` | `-50%` on a vertically centered placement. |
| `--lsx-z-index-float` | Theme stacking order for floats. |

## Data attributes

State tokens on the root's `data-state`, space separated, one per axis of
`placement`.

| Token | Condition |
|---|---|
| `vertical-top` / `vertical-center` / `vertical-bottom` | The vertical half of `placement`. |
| `horizontal-start` / `horizontal-center` / `horizontal-end` | The horizontal half of `placement`. |
