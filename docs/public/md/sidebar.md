# Sidebar

Crate: `libero`
Import: `use libero::components::Sidebar;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/sidebar.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An in-flow panel bordering one edge of its parent and scrolling its own content - a nav rail or inspector.

An in-flow panel bordering one edge of its parent, scrolling its own content -
a sidebar, nav rail or inspector. `side` picks the border and the size axis; the
panel's actual position is your layout's, so place it at the matching end of the
DOM. For the portaled, dimmed, focus-trapped kind, see [`Drawer`](drawer.md).

The content is wrapped in a [`ScrollArea`](scroll_area.md), so the panel scrolls
independently of the page. The panel itself is `flex-shrink: 0` - a sidebar is
usually a flex item that must not be squeezed.

## Usage

A `Sidebar` has no position of its own, so it needs a parent laying it out and a
sibling to sit beside. `side: "right"` means putting it *after* that sibling in
the DOM.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Flex, Sidebar, Text},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex {
            direction: "row",
            sx: sx().height("120px").width("100%").border("1px solid").border_color("grey.3"),
            Sidebar {
                Text { "Navigation" }
            }
            Flex {
                direction: "column",
                sx: sx().flex("1").padding("12px"),
                Text { "Rest of the layout" }
            }
        }
    }
}
```

The same panel on the right, at the other end of the DOM, with a wider `size`:

```rust
use dioxus::prelude::*;
use libero::{
    components::{Flex, Sidebar, Text},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex {
            direction: "row",
            sx: sx().height("120px").width("100%").border("1px solid").border_color("grey.3"),
            Flex {
                direction: "column",
                sx: sx().flex("1").padding("12px"),
                Text { "Rest of the layout" }
            }
            Sidebar {
                side: "right",
                size: "lg",
                Text { "Navigation" }
            }
        }
    }
}
```

`side: "top"` and `"bottom"` border the horizontal edges instead, and `size` then
means height - so the parent wants `direction: "column"`.

## Accessibility

The root is a plain `<div>` with no landmark role of its own, deliberately: what
the panel *is* depends on what you put in it. Wrap navigation content in a `<nav>`
(or pass `role`/`aria-label` through the attributes) so it is announced as a
landmark; a settings or inspector panel is better named with `aria-label` on a
`region`. The inner `ScrollArea` keeps the panel keyboard-scrollable.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `side` | `SidebarSide` | `left` | Which edge this panel borders and which axis `size` applies to. It does not place the panel - an in-flow item is positioned by its parent's layout, so put it at the matching end of the DOM yourself. |
| `size` | `Size` | `md` | The panel's width (or height, on a `top`/`bottom` side). |
| `children` | `Element` | required | The panel's content, scrolled by an inner `ScrollArea`. |

`SidebarSide` is `left`, `right`, `top` or `bottom`.

Like every component, `Sidebar` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`SidebarDefaults` on the theme. The default `side` (`left`) and `size` (`md`) are
the enum's own default and a hardcoded value, not theme fields.

| Field | Type | Description |
|---|---|---|
| `size` | `Sizes<u16>` | Panel extent in px per size step - 200, 240, 280, 320, 400, 480. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-sidebar-size-<size>` | Panel width (or height) for that size step. |

The 1px edge border reads the shared grey scale at shade 4.

## Data attributes

State tokens on the root's `data-state`, space separated. `side` and `size` are
read together - the pair decides whether the size lands on `width` or `height`.

| Token | Condition |
|---|---|
| `side-left` / `side-right` / `side-top` / `side-bottom` | The `side` in effect; also picks which edge gets the border. |
| `size-<size>` | The `size` in effect. |
