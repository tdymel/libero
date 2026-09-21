# Header

Crate: `libero`
Import: `use libero::components::Header;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/header.rs>
Index: [index.md](index.md) lists every other page
Description: The page's banner landmark, a sticky, static or fixed `header` bar for nav and actions.

The page's banner landmark, always a `header` element. This site's own header is
one. `sticky`, the default, pins to the top of the scrolling ancestor and
`static` scrolls away. `fixed` pins to the viewport, so offset your content by
`var(--lsx-header-height)`.

The header is at least `size` tall and grows when its content wraps. With
`publish_height`, a `sticky` or `fixed` header publishes its `size` on `:root`
as `--lsx-header-height`, for whatever sits below it. Set it on the page's own
banner only. With more than one, the last mounted wins.

## Usage

Inside a scrolling frame, so the positions differ. The frame is a stacking
context (`position` plus `z-index`), so a native window clips the header to it
too:

```rust
use dioxus::prelude::*;
use libero::components::{Box, Header, Text};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().height("200px").width("100%").overflow_y("auto")
                .position("relative").z_index("0")
                .border("1px solid var(--lsx-muted-3)"),
            Header { color: "primary", "Libero" }
            Box {
                sx: sx().padding("md"),
                for i in 0..12 {
                    Text { key: "{i}", "Scroll me, line {i}" }
                }
            }
        }
    }
}
```

The page's own banner:

```rust
use dioxus::prelude::*;
use libero::components::Header;

#[component]
fn Demo() -> Element {
    rsx! {
        Header { publish_height: true, "Libero" }
    }
}
```

## Accessibility

### Libero handles

- A `sticky` or `fixed` header with `publish_height` sets
  `scroll-padding-top: var(--lsx-header-height)` on `:root`, so focus moved
  under it scrolls clear (WCAG 2.4.11). That pads the page's scroller only.

### You must

- Keep the page's header at the top level, outside `main`, `nav`, `section`,
  `article` and `aside`: only there is it the `banner` landmark. One banner
  per page.
- Put a `nav` inside it for the navigation landmark.
- Give a `sticky` or `fixed` header `publish_height`, so focus scrolls clear
  of it.
- Give a header stuck inside another scroller the same `scroll-padding-top` on
  that scroller.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `position` | `HeaderPosition` | `sticky` | `sticky` pins to the top of the scrolling ancestor, `static` scrolls away. `fixed` pins to the viewport, so offset your content by `var(--lsx-header-height)`. |
| `size` | `ThemeAwareValue` | `md` | Minimum height. The header grows when its content wraps. |
| `color` | `ThemeAwareValue` | none, a neutral background | Fills the header with shade 6 and a readable text color. Under a gradient, its first stop. |
| `gradient` | `Gradient` | - | Fills the header with a gradient from `color`, as on `Paper`: `("info", 90)` or `Gradient::default().to("info").deg(90)`; `Gradient::default()` is the theme's. The text colour and focus rings are picked to read on both stops. |
| `z_index` | `ThemeAwareValue` | `100` | Stacking order. |
| `publish_height` | `bool` | `false` | Publishes the height as `--lsx-header-height` and `scroll-padding-top` on `:root`, so focus scrolls clear of a sticky or fixed banner. Set it on the page's own banner only. |
| `children` | `Element` | required | Nav and actions. |

Like every component, `Header` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`HeaderDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `heights` | `Sizes<u16>` | Header height in pixels per size step. |

The stacking order comes from the theme's shared `z_index` scale.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-header-height-<size>` | Height for that size step. |
| `--lsx-header-height` | The header's minimum height, from `size`. With `publish_height` it is also set on `:root`. |
| `--lsx-header-background` | Background of a colored header. Unset leaves it neutral. |
| `--lsx-header-color` | Text color of a colored header. Unset inherits. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `static` | `position` is `static`. |
| `fixed` | `position` is `fixed`. |

`sticky` is the default and has no token.
