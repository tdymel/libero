# ScrollArea

Crate: `libero`
Import: `use libero::components::{ScrollArea, ScrollPositionEvent};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroll_area.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A scrollable region that fills its parent, with themed scrollbars, percent-based scroll positions and per-edge events.

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

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-scroll-area-thumb-color` | Set by `scrollbar_color`; falls back to `grey.5`. The track is always transparent. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `axis-vertical` / `axis-horizontal` / `axis-both` / `axis-none` | The `scrollbars` axes in effect. |
| `visible-always` / `visible-hover` / `visible-hidden` | The `scrollbar_visibility` in effect - `scroll` writes `visible-hover`. |
| `size-thin` / `size-auto` | The `scrollbar_size` in effect. |
