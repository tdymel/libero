# ScrollArea

Crate: `libero`
Import: `use libero::components::{ScrollArea, ScrollPositionEvent, Virtualize};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroll_area/scroll_area.rs>
Index: [index.md](index.md) lists every other page
Description: A scrollable region that fills its parent, with themed scrollbars, scroll positions in percent, edge events and row virtualization through `Virtualize`.

Scrolls its content and fills its parent, so give the parent a size. `onscroll`
reports the position as a percent of each axis, and each edge has its own event.
The buttons scroll through a handle from `use_scroll_area()`.

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
            Text { size: "sm", "{readout(position())}" }
            // A reached edge is a result, so a screen reader hears it.
            Text { size: "sm", role: "status", "Last edge: {edge()}" }
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

With virtualize on, the rows come from `Virtualize`:

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
                    item: move |index: usize| rsx! {
                        ListItem {
                            aria_setsize: rows.read().len(),
                            aria_posinset: index + 1,
                            "{rows.read()[index]}"
                        }
                    },
                }
            }
        }
    }
}
```

## Virtualize

`Virtualize` renders only the rows in view out of 50,000. It draws no element of
its own and needs a `ScrollArea` above it, one per area. Without one, or as the
second, it warns and renders every row.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `scrollbars` | `ScrollAxis` | `vertical` | Which axes scroll and show a scrollbar. `none` clips the overflow. |
| `scrollbar_visibility` | `ScrollbarVisibility` | `always` | When the scrollbar shows, `always`, `hover` or `hidden`. `scroll` acts like `hover` for now. In a browser or WebView, `always` draws its own track and thumb, so the bar stays where the system overlays and fades its scrollbars; drag the thumb or press the track. The other values keep the native bar. |
| `scrollbar_size` | `ScrollbarSize` | `thin` | `thin` or `auto`: the CSS `scrollbar-width`, or 8px and 12px for the bar `always` draws. |
| `scrollbar_color` | `ThemeAwareValue` | - | Thumb color. The track stays transparent. Unset it is `muted.6`. |
| `offset_scrollbars` | `bool` | `false` | Pads the end edge by the drawn bar's width, 8px or 12px, so a flush trailing control is not under it. Only for the bar `always` draws; it pads even while the content fits, so the layout does not shift when it starts to overflow. |
| `scroll_position_x` | `f64` | - | Scrolls to this percent (0-100) horizontally. A signal re-applies it on every change, a literal once at mount. |
| `scroll_position_y` | `f64` | - | The same as `scroll_position_x`, vertically. |
| `handle` | `ScrollAreaHandle` | - | From `use_scroll_area()`. Its `scroll_to_percent(x, y)` and `scroll_to(x, y)` in px scroll the area from any handler, on every call. A `None` percent keeps that axis; a call before the area mounts does nothing. |
| `focusable` | `bool` | `false` | Makes the area a tab stop always. Without it, the area is one only while it overflows and holds nothing focusable. Your own `tabindex`, or a `role` other than `region`, turns that off. |
| `onscroll` | `EventHandler<ScrollPositionEvent>` | - | Fires on every scroll with the position as a percent of each axis: `Start` when a scroll begins, `Change` while it runs, `End` when it stops. |
| `onresize` | `EventHandler<Event<ResizeData>>` | - | Fires after the area resized. |
| `ontopreached` | `EventHandler<()>` | - | Fires once when the top edge is reached. |
| `onbottomreached` | `EventHandler<()>` | - | Fires once when the bottom edge is reached, and again when content added at the bottom is scrolled to its new end, say a list loading more. |
| `onleftreached` | `EventHandler<()>` | - | Fires once when the left edge is reached. |
| `onrightreached` | `EventHandler<()>` | - | Fires once when the right edge is reached. |
| `children` | `Element` | required | The scrollable content. |
| `parts` | `Parts<ScrollAreaPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

`Virtualize` renders no element, so it takes no styling props.

| Prop | Type | Default | Description |
|---|---|---|---|
| `count` | `usize` | required | Rows in the whole list, not only the rendered ones. |
| `item` | `Callback<usize, Element>` | required | Renders one row. Called only for the rows in view. |
| `item_size` | `f64` | measured | A row's height plus the gap below it, in px. Unset, it is measured from the first rows, and again when the area's width changes. Every row must have the same height. |
| `overscan` | `usize` | `4` | Rows rendered beyond each edge, so a fast scroll has something to show. |
| `keep_rendered` | `usize` | none | An index rendered even out of view, e.g. the row holding focus, so scrolling it away keeps the focus. |
| `item_key` | `Callback<usize, String>` | the index | A row's identity, such as its data's id. A row's state (focus, typed text, open details) follows its key, so set it when rows can be sorted, inserted or removed. |

Like every component, `ScrollArea` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

`ScrollPositionEvent` is `Start(x, y)`, `Change(x, y)` or `End(x, y)`, in
percent.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ScrollAreaPart::Scrollbar` | `scrollbar` | A track `always` draws in a browser. `data-orientation` is `vertical` or `horizontal`. |
| `ScrollAreaPart::Thumb` | `thumb` | The thumb inside a track. |

## Accessibility

### Libero handles

- Tab reaches focusable content inside the area as usual.
- When the content has nothing to focus, like a block of text, the area
  itself becomes a tab stop while it overflows, so the arrow keys can scroll
  it. The Usage example is one. `focusable: true` keeps the stop always.
- A debug build warns about a tab stop without a name.
- On Blitz, which scrolls nothing on a key, the area scrolls itself on the
  arrows, Page Up and Down, Home, End and Space, as a browser does.
- The bar `always` draws is hidden from screen readers and takes no focus:
  the area itself scrolls by keyboard, wheel and touch. Forced colours paint
  its thumb in the system text colour.

### You must

- Name the area with `aria_label` or `aria_labelledby`.
- Use `scrollbars: "none"` only where something else scrolls: it puts the
  clipped content out of reach.
- Give each `Virtualize` row `aria_setsize: count` and `aria_posinset: index + 1`,
  as the virtualize example does: only the rows in view exist, so a screen
  reader cannot count the rest. For a table row, `aria-rowcount` and
  `aria-rowindex`.
- Pass `focusable: true` when the content holds only controls hidden by CSS
  (`visibility: hidden`, `display: none`): the area counts them as focusable
  and makes no tab stop.
- Pad the content by 6 px or more where a focusable child sits flush with the
  area's edge: the area clips the outset focus ring there.
- Set `offset_scrollbars: true` where a control sits flush with the end edge,
  such as a trailing icon button in a flush row or a table's last-column
  action: the drawn bar covers the last 8 px (12 px at `auto`) and takes the
  click, which leaves a 24 px control under the 24 px target minimum.

### Example

A terms text in `ScrollArea { aria_label: "Terms of service", .. }`: Tab stops
on the area, a screen reader reads its name, and the arrow keys scroll it.

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
| `--lsx-scroll-area-thumb-color` | Thumb color, from `scrollbar_color` or `muted.6`. |
| `--lsx-scroll-area-leading` | Space for the rows a `Virtualize` skipped above the view. |
| `--lsx-scroll-area-trailing` | The same below the view. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `axis-vertical` / `axis-horizontal` / `axis-both` / `axis-none` | The `scrollbars` axes in effect. |
| `visible-always` / `visible-hover` / `visible-hidden` | The `scrollbar_visibility` in effect. `scroll` writes `visible-hover`. |
| `size-thin` / `size-auto` | The `scrollbar_size` in effect. |
| `offset-inline` / `offset-block` | `offset_scrollbars` pads the inline or block end. |

The content box inside the root carries `virtualized` once it holds a
`Virtualize`.

Under `always` in a browser, the root's first child is the drawn bar: an
`aria-hidden`, absolutely positioned `[data-slot="scrollbars"]` layer holding a
`[data-slot="scrollbar"]` track per axis, its `data-orientation` `vertical` or
`horizontal`, each with a `[data-slot="thumb"]`. It stays out of the content's flow, so a flex or grid root lays
out as without it. The root is `position: relative` then, a stacking context
for the layer.
