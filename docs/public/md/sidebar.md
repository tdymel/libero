# Sidebar

Crate: `libero`
Import: `use libero::components::Sidebar;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/sidebar.rs>
Index: [index.md](index.md) lists every other page
Description: An in-flow panel on one edge of its parent that scrolls its own content, like a nav rail or an inspector.

An in-flow panel on one edge of its parent that scrolls its own content. `side`
picks the border, not the position, so an end sidebar comes after its sibling
in the DOM. For a panel that slides in over the page, see
[`use_drawer`](drawer.md).

The content sits in a [`ScrollArea`](scroll_area.md), so it scrolls apart from
the page. The panel does not shrink in a flex row.

## Usage

A `Sidebar` needs a parent to lay it out and a sibling to sit beside.

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
            align: "stretch",
            wrap: false,
            sx: sx().height("120px").width("100%").border("1px solid").border_color("muted.3"),
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

The same panel at the end, after its sibling, with a wider `size`.

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
            align: "stretch",
            wrap: false,
            sx: sx().height("120px").width("100%").border("1px solid").border_color("muted.3"),
            Flex {
                direction: "column",
                sx: sx().flex("1").padding("12px"),
                Text { "Rest of the layout" }
            }
            Sidebar {
                side: "end",
                size: "lg",
                Text { "Navigation" }
            }
        }
    }
}
```

With `side: "top"` or `"bottom"`, `size` is a height, so the parent needs
`direction: "column"`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `side` | `SidebarSide` | `start` | The edge that gets the border, and whether `size` is a width or a height. It does not move the panel, so put it at the matching end of the DOM. |
| `size` | `Size` | `md` | The panel's width, or its height on a `top` or `bottom` side. |
| `component` | `HtmlTag` | `aside` | The element to render, such as `nav` for a navigation panel. |
| `children` | `Element` | required | The panel's content, scrolled by an inner `ScrollArea`. |
| `parts` | `Parts<SidebarPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

`SidebarSide` is `start`, `end`, `top` or `bottom`. `start` and `end` follow the text direction: `start` is the right edge under `dir="rtl"`.

Like every component, `Sidebar` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `SidebarPart::Scroll` | `scroll` | The `ScrollArea` holding the content. It carries the panel's padding. |

## Accessibility

### Libero handles

- The root is an `aside`, the `complementary` landmark.
- Content that overflows with nothing focusable in it makes the inner scroll
  area a tab stop, a `region` that takes the panel's `aria_label` or
  `aria_labelledby`.

### You must

- Pass `component: "nav"` for the site navigation.
- Give it an `aria_label` when the page has more than one landmark of that
  kind.
- Give it an `aria_label` or `aria_labelledby` when its content can overflow
  with nothing focusable in it, or the tab stop is an unnamed region.

### Example

The site navigation, `Sidebar { component: "nav", aria_label: "Main" }`: a
screen reader lists it as the "Main" navigation landmark.

## Theme defaults

`SidebarDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size`, `md`. |
| `sizes` | `Sizes<u16>` | Panel size in px per step: 200, 240, 280, 320, 400, 480. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-sidebar-size-<size>` | Panel width or height for that size step. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `side-start` / `side-end` / `side-top` / `side-bottom` | The `side` in effect. |
| `size-<size>` | The `size` in effect. |
