# ScrollArea

Crate: `libero`
Import: `use libero::components::{ScrollArea, ScrollPositionEvent, Virtualize};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroll_area.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A scrollable region that fills its parent, with themed scrollbars, percent-based scroll positions, per-edge events, and row virtualization through `Virtualize`.

Scrolls its content, filling the parent by default. `on_scroll` reports the
position as a percent of each axis - `Start`/`End` bracket one scroll, `Change`
carries the rest - and `scroll_position_x`/`scroll_position_y` scroll it to a
percent. Each edge has its own event: `on_top_reached`, `on_bottom_reached`,
`on_left_reached`, `on_right_reached`.

It is `width: 100%; height: 100%`, so it needs a parent with a real size - inside
a box that shrink-wraps its content there is nothing to scroll.

## Usage

The frame around it is the example: the area fills its parent, the readout and
the two jump buttons are the other half of the wiring.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex, ScrollArea, ScrollPositionEvent, Text},
    sx::sx,
};

fn readout(event: ScrollPositionEvent) -> String {
    let (kind, x, y) = match event {
        ScrollPositionEvent::Start(x, y) => ("Start", x, y),
        ScrollPositionEvent::Change(x, y) => ("Change", x, y),
        ScrollPositionEvent::End(x, y) => ("End", x, y),
    };
    format!("{kind} at x {x:.0}%, y {y:.0}%")
}

#[component]
fn Demo() -> Element {
    let mut position = use_signal(|| ScrollPositionEvent::Change(0.0, 0.0));
    let mut edge = use_signal(|| "none");
    let mut jump = use_signal(|| None::<f64>);

    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            sx: sx().width("100%"),
            Box {
                sx: sx().height("160px").width("100%").border("1px solid var(--lsx-grey-3)"),
                ScrollArea {
                    scroll_position_y: jump(),
                    on_scroll: move |event: ScrollPositionEvent| position.set(event),
                    on_top_reached: move |_| { edge.set("top"); jump.set(None) },
                    on_bottom_reached: move |_| { edge.set("bottom"); jump.set(None) },
                    Box {
                        sx: sx().width("150%").padding("md"),
                        for i in 0..20 {
                            Text { key: "{i}", "Item {i}" }
                        }
                    }
                }
            }
            Text { size: "sm", "{readout(position())} - last edge: {edge()}" }
            Flex {
                gap: "sm",
                Button { size: "sm", variant: "outlined", onclick: move |_| jump.set(Some(0.0)), "Scroll to top" }
                Button { size: "sm", variant: "outlined", onclick: move |_| jump.set(Some(100.0)), "Scroll to bottom" }
            }
        }
    }
}
```

`scroll_position_x`/`scroll_position_y` are percents (0-100). Bound to a signal
they re-apply on every change, so clearing the signal back to `None` in the
matching `on_*_reached` is what lets the same button be pressed twice; a literal
applies once, at mount.

## Virtualization

`Virtualize` renders only the rows the area can show, so a fifty-thousand-row
list costs about a dozen elements. It draws no element of its own, so it goes
wherever the rows go - inside a `List`, a table body, a plain stack - and it
needs a `ScrollArea` above it. That is the whole contract: the `ScrollArea`
tells it where the viewport is, and pads itself with the space the skipped rows
would have taken, so the scrollbar still spans the whole list.

```rust
use dioxus::prelude::*;
use libero::components::{List, ListItem, ScrollArea, Virtualize};

#[component]
fn Rows() -> Element {
    let rows = use_signal(|| (0..50_000).map(|i| format!("Row {i}")).collect::<Vec<_>>());

    rsx! {
        ScrollArea {
            List {
                Virtualize {
                    count: rows.read().len(),
                    item: move |index| rsx! {
                        ListItem { "{rows.read()[index]}" }
                    },
                }
            }
        }
    }
}
```

`count` and `item` rather than the rows themselves: the closure reads the list
where it lives, so nothing is cloned into props and only the visible rows are
ever touched.

Rows must be a uniform height. `Virtualize` measures the first rows it renders -
once one row, then eight, so the difference gives the row's height plus the gap
below it and whatever else shares the scroll area cancels out - and assumes the
rest match. Pass `item_size` in px to skip the measurement, which off the web
saves two round-trips.

Without a `ScrollArea` above it, `Virtualize` warns and renders every row -
correct, just not virtualized. One `Virtualize` per `ScrollArea`: they would
otherwise fight over the same padding, so a second one warns and renders every
row too.

## Accessibility

The root is a `<div>` with `tabindex="-1"`: Chromium makes an overflowing
`overflow: auto` region an implicit tab stop, which would add a stop that
announces nothing, and the content carries its own focusable elements. Keyboard
scrolling still works once something inside has focus, and
`scrollbar_visibility: "hover"` also reveals the bar on `:focus-within` so a
keyboard user is not left without one.

`scrollbars: "none"` hides overflow on both axes - content outside the box becomes
unreachable, so use it only when something else provides the scrolling.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `scrollbars` | `ScrollAxis` | `vertical` | Which axes show a scrollbar and allow overflow: `vertical`, `horizontal`, `both` or `none`. |
| `scrollbar_visibility` | `ScrollbarVisibility` | `always` | `always`, `hover`, `hidden`, or `scroll` (currently identical to `hover`). |
| `scrollbar_size` | `ScrollbarSize` | `thin` | CSS `scrollbar-width`: `thin` or `auto`. |
| `scrollbar_color` | `ThemeAwareValue` | - | Scrollbar thumb color - the track stays transparent. Unset it is `grey.5`. |
| `scroll_position_x` | `f64` | - | Percent (0-100) to scroll to horizontally. Bound to a signal it re-applies on every change; a literal applies once, at mount. |
| `scroll_position_y` | `f64` | - | Percent (0-100) along the vertical axis - see `scroll_position_x`. |
| `on_scroll` | `EventHandler<ScrollPositionEvent>` | - | Fires on every scroll tick with the position as a percent of each axis's scrollable range. |
| `on_top_reached` | `EventHandler<()>` | - | Fires once when the top edge is reached. |
| `on_bottom_reached` | `EventHandler<()>` | - | Fires once when the bottom edge is reached. |
| `on_left_reached` | `EventHandler<()>` | - | Fires once when the left edge is reached. |
| `on_right_reached` | `EventHandler<()>` | - | Fires once when the right edge is reached. |
| `children` | `Element` | required | The scrollable content. |

`Virtualize` takes no styling props - it renders no element of its own.

| Prop | Type | Default | Description |
|---|---|---|---|
| `count` | `usize` | required | Rows in the whole list, not just the rendered ones. |
| `item` | `Callback<usize, Element>` | required | Renders one row. Called only for the rows in view. |
| `item_size` | `f64` | measured | Row pitch in px - a row's height plus the gap below it. |
| `overscan` | `usize` | `theme.scroll_area.overscan` | Rows kept beyond each edge, so a scroll has something to reveal before the next render lands. |

Like every component, `ScrollArea` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

`ScrollPositionEvent` is `Start(x, y)`, `Change(x, y)` or `End(x, y)`, each in
percent of that axis's scrollable range.

## Theme defaults

`ScrollAreaDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `scrollbars` | `ScrollAxis` | Default axes (`vertical`). |
| `visibility` | `ScrollbarVisibility` | Default visibility (`always`). |
| `size` | `ScrollbarSize` | Default `scrollbar-width` keyword (`thin`). |
| `overscan` | `usize` | Rows a `Virtualize` keeps beyond each edge (`4`). |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-scroll-area-thumb-color` | Set by `scrollbar_color`; falls back to `grey.5`. The track is always transparent. |
| `--lsx-scroll-area-leading` | Space standing in for the rows a `Virtualize` skipped above the window, on the content box rather than the container. Unset outside virtualization. |
| `--lsx-scroll-area-trailing` | The same below the window. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `axis-vertical` / `axis-horizontal` / `axis-both` / `axis-none` | The `scrollbars` axes in effect. |
| `visible-always` / `visible-hover` / `visible-hidden` | The `scrollbar_visibility` in effect - `scroll` writes `visible-hover`. |
| `size-thin` / `size-auto` | The `scrollbar_size` in effect. |

The content box inside the root carries `virtualized` once a `Virtualize` claims
it. Until then it is `display: contents`, so it has no box of its own and an
ordinary scroll area lays out as if it were not there.
