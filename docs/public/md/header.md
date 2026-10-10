# Header

Crate: `libero`
Import: `use libero::components::Header;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/header.rs>
Index: [index.md](index.md) lists every other page
Description: The page's banner landmark, a sticky, static or fixed `header` bar for nav and actions.

The page's banner landmark, always a `header` element. This site's own header is
one. `sticky`, the default, pins to the top of the scrolling ancestor and
`static` scrolls away. `fixed` pins to the viewport, so offset your content by
`var(--lsx-header-height)`. In a native app `fixed` scrolls with the page for
now; use `sticky`.

The header is at least `size` tall and grows when its content wraps. With
`publish_height`, a `sticky` or `fixed` header publishes its `size` on `:root`
as `--lsx-header-height`, for whatever sits below it. Set it on the page's own
banner only. With more than one, the last mounted wins.

## Usage

Inside a named `ScrollArea`, so the positions differ and the frame scrolls by
keyboard on every renderer. The frame is a stacking context (`position` plus
`z-index`), so a native window clips the header to it too:

```rust
use dioxus::prelude::*;
use libero::components::{Box, Header, ScrollArea, Text};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().height("200px").width("100%")
                .position("relative").z_index("0")
                .border("1px solid var(--lsx-muted-3)"),
            ScrollArea {
                aria_label: "Header demo",
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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `position` | `HeaderPosition` | `sticky` | `sticky` pins to the top of the scrolling ancestor, `static` scrolls away. `fixed` pins to the viewport, so offset your content by `var(--lsx-header-height)`. In a native app `fixed` scrolls with the page for now; use `sticky`. |
| `size` | `ThemeAwareValue` | `md` | Minimum height. The header grows when its content wraps. |
| `color` | `ThemeAwareValue` | none, a neutral background | Fills the header with shade 6 and a readable text color. A literal CSS color is used as given, with black or white text, links and focus rings, whichever reads better on it. With `glass`, a translucent tint of it, as on `Paper`. Under a gradient, its first stop. |
| `glass` | `bool` | `false` | Frosted glass, as on `Paper`: content scrolling under the bar shows through, blurred. A `color` tints it, as on `Paper`. Opaque when the user reduces transparency, in forced colours, and in native windows. |
| `gradient` | `Gradient` | - | Fills the header with a gradient from `color`, as on `Paper`: `("info", 90)` or `Gradient::default().to("info").deg(90)`; `Gradient::default()` is the theme's. The text colour and focus rings are picked to read on both stops. With `glass`, the stops turn translucent. |
| `z_index` | `ThemeAwareValue` | `100` | Stacking order. |
| `publish_height` | `bool` | `false` | Publishes the height as `--lsx-header-height` and `scroll-padding-top` on `:root`, so focus scrolls clear of a sticky or fixed banner. Set it on the page's own banner only. |
| `children` | `Element` | required | Nav and actions. |

Like every component, `Header` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- Draws a `header` element, which screen readers list as the page's banner
  when it sits at the top level.
- With `publish_height`, an element that gets focus under a `sticky` or
  `fixed` header scrolls clear of it (WCAG 2.4.11). This covers the page's own
  scroller only.

### You must

- Put the page's header at the top level, not inside `main`, `nav`, `section`,
  `article` or `aside`. Use one per page.
- Put a `nav` inside it for the site's links.
- Set `publish_height` on a `sticky` or `fixed` header.
- For a header stuck inside another scroller, set `scroll-padding-top:
  var(--lsx-header-height)` on that scroller yourself.

### Example

A page banner: `Header { publish_height: true, nav { .. } }` as the app's
first child, outside `main`. A screen reader lists a banner with the
navigation in it, and a link Tab focuses near the top scrolls into view
instead of under the bar.

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
| `glass` | `glass` is set and the renderer draws a backdrop blur (not in native windows). |
| `colored` | `color` is set without a `gradient`. |
| `gradient` | `gradient` is set. |

`sticky` is the default and has no token.
