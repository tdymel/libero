# ScrollArea

Crate: `libero`
Import: `use libero::components::{ScrollArea, ScrollPositionEvent, Virtualize};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroll_area/scroll_area.rs>
Index: [index.md](index.md) lists every other page
Description: A scrollable region that fills its parent, with themed scrollbars, scroll positions in percent, edge events and row virtualization through `Virtualize`.

Scrolls its content and fills its parent, so give the parent a size. `onscroll`
reports the position as a percent of each axis, and each edge has its own event.
`Virtualize` renders only the rows in view.

## Usage

The `Box` gives the area its size. The readout and the buttons show the events
and the handle.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex, ScrollArea, ScrollPositionEvent, Text, use_scroll_area},
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
    let area = use_scroll_area();

    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            sx: sx().width("100%"),
            Box {
                sx: sx().height("160px").width("100%").border("1px solid var(--lsx-muted-3)"),
                ScrollArea {
                    aria_label: "Items",
                    handle: area,
                    onscroll: move |event: ScrollPositionEvent| position.set(event),
                    ontopreached: move |_| edge.set("top"),
                    onbottomreached: move |_| edge.set("bottom"),
                    Box {
                        sx: sx().width("150%").padding("md"),
                        for i in 0..20 {
                            Text { key: "{i}", "Item {i}" }
                        }
                    }
                }
            }
            Text { size: "sm", "{readout(position())}, last edge: {edge()}" }
            Flex {
                gap: "sm",
                Button { size: "sm", variant: "outlined", onclick: move |_| area.scroll_to_percent(None, Some(0.0)), "Scroll to top" }
                Button { size: "sm", variant: "outlined", onclick: move |_| area.scroll_to_percent(None, Some(100.0)), "Scroll to bottom" }
                Button { size: "sm", variant: "outlined", onclick: move |_| area.scroll_to(0.0, 120.0), "Scroll to 120px" }
            }
        }
    }
}
```

`ScrollPositionEvent` is `Start` when a scroll begins, `Change` while it runs
and `End` when it stops, each with the x and y percent.

`use_scroll_area()` returns a `Copy` handle. Pass it as `handle`, then call
`scroll_to_percent(x, y)` (0-100, `None` keeps that axis) or `scroll_to(x, y)`
in px from any handler. Every call scrolls. A call before the area mounts does
nothing.

`scroll_position_x` and `scroll_position_y` scroll to a percent too. A signal
re-applies only when its value changes, so asking for the same position twice
does nothing. Use the handle for buttons.

## Virtualize

`Virtualize` renders only the rows the area can show, so a list of 50,000 rows
costs about a dozen elements. It draws no element of its own, so it goes
wherever the rows go, inside a `List`, a table body or a plain stack. It needs a
`ScrollArea` above it, which keeps the scrollbar the length of the whole list.

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

It takes a `count` and an `item` closure rather than the rows, so only the
visible rows are read.

Every row must have the same height. `Virtualize` measures the first rows and
assumes the rest match. Pass `item_size` in px to skip the measurement.

Without a `ScrollArea` above it, `Virtualize` warns and renders every row. A
`ScrollArea` holds one `Virtualize`. A second one warns and renders every row
too.

## Accessibility

Tab reaches focusable content inside the area as usual. When the content has
nothing to focus, like a block of text, the area itself becomes a tab stop with
`role="region"` while it overflows, so the arrow keys can scroll it.
`focusable: true` keeps the stop always. Your own `tabindex`, or a `role` other
than `region`, turns it off.

Name the area with `aria_label` or `aria_labelledby`. A debug build warns about
a tab stop without a name. The name applies only while the area has a role.

```rust,ignore
ScrollArea {
    aria_label: "Release notes",
    Text { "..." }
}
```

`scrollbars: "none"` clips the overflow on both axes, so the content outside the
box is out of reach. Use it only when something else scrolls.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `scrollbars` | `ScrollAxis` | `vertical` | Which axes scroll and show a scrollbar. `none` clips the overflow. |
| `scrollbar_visibility` | `ScrollbarVisibility` | `always` | When the scrollbar shows, `always`, `hover` or `hidden`. `scroll` acts like `hover` for now. |
| `scrollbar_size` | `ScrollbarSize` | `thin` | The CSS `scrollbar-width`, `thin` or `auto`. |
| `scrollbar_color` | `ThemeAwareValue` | - | Thumb color. The track stays transparent. Unset it is `muted.5`. |
| `scroll_position_x` | `f64` | - | Scrolls to this percent (0-100) horizontally. A signal re-applies it on every change, a literal once at mount. |
| `scroll_position_y` | `f64` | - | The same as `scroll_position_x`, vertically. |
| `handle` | `ScrollAreaHandle` | - | From `use_scroll_area()`. Its `scroll_to_percent(x, y)` and `scroll_to(x, y)` in px scroll the area from any handler, on every call. |
| `focusable` | `bool` | `false` | Makes the area a tab stop always. Without it, the area is one only while it overflows and holds nothing focusable. Your own `tabindex`, or a `role` other than `region`, turns that off. |
| `onscroll` | `EventHandler<ScrollPositionEvent>` | - | Fires on every scroll with the position as a percent of each axis. |
| `onresize` | `EventHandler<Event<ResizeData>>` | - | Fires after the area resized. |
| `ontopreached` | `EventHandler<()>` | - | Fires once when the top edge is reached. |
| `onbottomreached` | `EventHandler<()>` | - | Fires once when the bottom edge is reached. |
| `onleftreached` | `EventHandler<()>` | - | Fires once when the left edge is reached. |
| `onrightreached` | `EventHandler<()>` | - | Fires once when the right edge is reached. |
| `children` | `Element` | required | The scrollable content. |

`Virtualize` renders no element, so it takes no styling props.

| Prop | Type | Default | Description |
|---|---|---|---|
| `count` | `usize` | required | Rows in the whole list, not only the rendered ones. |
| `item` | `Callback<usize, Element>` | required | Renders one row. Called only for the rows in view. |
| `item_size` | `f64` | measured | A row's height plus the gap below it, in px. Unset, it is measured from the first rows. Every row must have the same height. |
| `overscan` | `usize` | `4` | Rows rendered beyond each edge, so a fast scroll has something to show. |

Like every component, `ScrollArea` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

`ScrollPositionEvent` is `Start(x, y)`, `Change(x, y)` or `End(x, y)`, in
percent.

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
| `--lsx-scroll-area-thumb-color` | Thumb color, from `scrollbar_color` or `muted.5`. |
| `--lsx-scroll-area-leading` | Space for the rows a `Virtualize` skipped above the view. |
| `--lsx-scroll-area-trailing` | The same below the view. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `axis-vertical` / `axis-horizontal` / `axis-both` / `axis-none` | The `scrollbars` axes in effect. |
| `visible-always` / `visible-hover` / `visible-hidden` | The `scrollbar_visibility` in effect. `scroll` writes `visible-hover`. |
| `size-thin` / `size-auto` | The `scrollbar_size` in effect. |

The content box inside the root carries `virtualized` once it holds a
`Virtualize`.
