# Header

Crate: `libero`
Import: `use libero::components::Header;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/header.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The page's banner landmark - a sticky, static or fixed `header` bar hosting nav and actions.

The page's banner landmark - always renders `header`. This docs site's own header
uses one. A set `color` takes shade 6 and picks its own contrast text.

`sticky` (the default) pins to the top of the scrolling ancestor, `static`
scrolls away with the content, and `fixed` is viewport-relative - with `fixed`
you offset your own content by `var(--lsx-header-height)`.

The banner is at least its `size` tall and grows when its content wraps. A
`sticky` or `fixed` header publishes that height on `:root` as
`--lsx-header-height`, for whatever is positioned below it. Nothing is measured:
it is the `size`, not the rendered height. With more than one, the last mounted
publishes, and unmounting it hands the root back to the one before.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Header;

#[component]
fn Demo() -> Element {
    rsx! {
        Header { position: "sticky", size: "md", color: "primary", "Libero" }
    }
}
```

Inside a scrolling frame, so the difference between the positions shows:

```rust
use dioxus::prelude::*;
use libero::components::{Box, Header, Text};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().height("200px").width("100%").overflow_y("auto")
                .border("1px solid var(--lsx-muted-3)"),
            Header { position: "sticky", "Libero" }
            Box {
                sx: sx().padding("md"),
                for i in 0..12 {
                    Text { key: "{i}", "Scroll me - line {i}" }
                }
            }
        }
    }
}
```

## Accessibility

It is always a `<header>`, which is the `banner` landmark only when it is not
nested inside `article`, `aside`, `main`, `nav` or `section` - so keep it at the
top level of the page. Put a `nav` inside it for the navigation landmark. One
banner per page.

A `sticky` or `fixed` header sets `scroll-padding-top: var(--lsx-header-height)`
on `:root`, so focus moved under it is scrolled clear (WCAG 2.4.11). That pads
the page's scroller only: a header stuck inside another scroller needs the same
padding on that scroller, from you.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `position` | `HeaderPosition` | `sticky` | `sticky` needs no offset; `fixed` is viewport-relative: offset your content by `var(--lsx-header-height)`. `static` scrolls away. |
| `size` | `ThemeAwareValue` | `md` | Header minimum height. |
| `color` | `ThemeAwareValue` | unset - neutral background, inherited text | A set color takes shade 6 and picks its own contrast text. |
| `z_index` | `ThemeAwareValue` | `100` | Stacking order. |
| `children` | `Element` | required | Nav and actions, hosted rather than scoped. |

Like every component, `Header` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`HeaderDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `heights` | `Sizes<u16>` | Header height in pixels per size step. |

The stacking order comes from the theme's shared `z_index` scale, not from
`HeaderDefaults`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-header-height-<size>` | Height for that size step. |
| `--lsx-header-height` | The header's minimum height, from `size`. A `sticky` or `fixed` header also publishes it on `:root`, as the offset for things below it. |
| `--lsx-header-height-override` | The `size` prop, set per instance; wins over the size step. |
| `--lsx-header-background` | Background of a colored banner; unset leaves it white. |
| `--lsx-header-color` | Contrast text color of a colored banner; unset inherits. |
| `--lsx-z-index-header-override` | The `z_index` prop, set per instance. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `static` | `position` is `static`. |
| `fixed` | `position` is `fixed`. |

`sticky` is the base rule, so it has no token. The publishing header's id is on
the root as `data-lsx-header`.
