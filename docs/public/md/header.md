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
you offset your own content, as with `Drawer`'s anchor.

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
                .border("1px solid var(--lsx-grey-3)"),
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

It is always a `<header>` element, which is the `banner` landmark when it is not
nested inside `article`, `aside`, `main`, `nav` or `section` - so keep it at the
top level of the page. It hosts nav and actions rather than scoping them: put a
`nav` inside it for the navigation landmark. One banner per page.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `position` | `HeaderPosition` | `sticky` | `sticky` needs no offset; `fixed` is viewport-relative and you offset your own content, as with `Drawer`'s `anchor`. `static` scrolls away. |
| `size` | `ThemeAwareValue` | `md` | Header height. |
| `color` | `ThemeAwareValue` | unset - neutral background, inherited text | A set color takes shade 6 and picks its own contrast text. |
| `z_index` | `ThemeAwareValue` | `100` | Stacking order. |
| `children` | `Element` | required | Nav and actions, hosted rather than scoped. |

Like every component, `Header` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`HeaderDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `height` | `Sizes<u16>` | Header height in pixels per size step. |

The stacking order comes from the theme's shared `z_index` scale, not from
`HeaderDefaults`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-header-height-<size>` | Height for that size step. |
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

`sticky` is the base rule, so it has no token.
