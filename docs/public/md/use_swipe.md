# Swipe

Crate: `libero`
Import: `use libero::hooks::{EdgeSwipeOptions, Swipe, SwipeDirection, SwipeEdge, SwipeEvent, SwipeOptions, edge_swipe_sx, use_edge_swipe, use_swipe};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/swipe.rs>
Index: [index.md](index.md) lists every other page
Description: Touch swipe handlers, and an edge swipe that opens a drawer without fighting Android's back gesture.

`use_swipe(on_swipe, options) -> Swipe` calls `on_swipe` once a touch or pen
moves `options.distance` px (48) along one axis, more than along the other. It
fires during the move, once per press, with the `direction`, the start point and
the delta. Spread `onpointerdown`, `onpointermove`, `onpointerup` and
`onpointercancel` onto one element, and give it a `touch-action`: `none` for all
four directions, `pan-y` for sideways swipes on a page that scrolls.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, ButtonGroup, Flex},
    hooks::{SwipeDirection, SwipeEvent, SwipeOptions, use_swipe},
    sx::sx,
};

#[component]
fn SwipePad() -> Element {
    let mut last = use_signal(|| None::<SwipeDirection>);
    let swipe = use_swipe(
        Callback::new(move |event: SwipeEvent| last.set(Some(event.direction))),
        SwipeOptions::default(),
    );
    let said = match last() {
        Some(direction) => format!("Swiped {direction:?}"),
        None => "No swipe yet".to_string(),
    };

    rsx! {
        Flex { direction: "column", align: "stretch", gap: "sm",
            Box {
                // All four directions reach the hook; the page does not scroll from here.
                sx: sx()
                    .touch_action("none")
                    .height("160px")
                    .border("1px dashed")
                    .border_radius("md")
                    .display("grid")
                    .place_items("center"),
                onpointerdown: move |event| swipe.onpointerdown.call(event),
                onpointermove: move |event| swipe.onpointermove.call(event),
                onpointerup: move |event| swipe.onpointerup.call(event),
                onpointercancel: move |event| swipe.onpointercancel.call(event),
                "Swipe here with a finger or a pen"
            }
            // The same choices without a gesture (WCAG 2.5.1).
            ButtonGroup { "aria-label": "Swipe by button",
                Button { onclick: move |_| last.set(Some(SwipeDirection::Left)), "Left" }
                Button { onclick: move |_| last.set(Some(SwipeDirection::Right)), "Right" }
            }
            div { role: "status", "{said}" }
        }
    }
}
```

An edge swipe that opens a navigation drawer, next to the burger that stays:

```rust
use dioxus::prelude::*;
use libero::{
    components::Box,
    hooks::{EdgeSwipeOptions, edge_swipe_sx, use_edge_swipe},
};

#[component]
fn Page(open: Signal<bool>, children: Element) -> Element {
    let mut open = open;
    let swipe = use_edge_swipe(
        Callback::new(move |()| open.set(true)),
        EdgeSwipeOptions::default(),
    );

    rsx! {
        Box {
            sx: edge_swipe_sx(),
            onpointerdown: move |event| swipe.onpointerdown.call(event),
            onpointermove: move |event| swipe.onpointermove.call(event),
            onpointerup: move |event| swipe.onpointerup.call(event),
            onpointercancel: move |event| swipe.onpointercancel.call(event),
            {children}
        }
    }
}
```

## Edge swipes

`use_edge_swipe(on_swipe, options) -> Swipe` is built on it: an inward swipe that
starts in a band near the start edge (the right edge under RTL) opens your
drawer. The band begins `inset` px (44) in, past Android 16's system back zone
at its highest sensitivity, and is `width` px (48) wide, so it needs no gesture
exclusion.
Spread it with `edge_swipe_sx()` on a box covering the page: no strip lies over
the content. A swipe from the band that goes inward, more sideways than up or
down, also starts on a code block or another inner scroller: the hook keeps that
box from scrolling sideways for the press. These docs open their navigation this
way on a phone, and the drawer follows the finger: past a third of its width or
on a flick it opens.

## API

```rust,ignore
pub fn use_swipe(on_swipe: Callback<SwipeEvent>, options: SwipeOptions) -> Swipe
pub fn use_edge_swipe(on_swipe: Callback, options: EdgeSwipeOptions) -> Swipe
pub fn edge_swipe_sx() -> Sx

pub struct SwipeOptions {
    pub distance: f64,
}

pub struct EdgeSwipeOptions {
    pub edge: SwipeEdge,
    pub inset: f64,
    pub width: f64,
    pub distance: f64,
}

pub struct SwipeEvent {
    pub direction: SwipeDirection,
    pub start: DragPoint,
    pub delta: DragPoint,
}

pub enum SwipeDirection { Left, Right, Up, Down }
pub enum SwipeEdge { Start, End }
```

| Option | Default | Description |
|---|---|---|
| `distance` | `48.0` | How far, in CSS px, the pointer travels along one axis (inward for an edge swipe) before it counts. |
| `edge` | `SwipeEdge::Start` | The inline edge an edge swipe starts from: left, right under RTL. `End` mirrors it. |
| `inset` | `44.0` | Where the band begins, CSS px in from the viewport's edge: past Android's system back zone. |
| `width` | `48.0` | How wide the band is, in CSS px. |

| Field of `Swipe` | Type | Description |
|---|---|---|
| `onpointerdown` | `Callback<PointerEvent>` | Starts tracking a touch or pen; a second finger drops it. |
| `onpointermove` | `Callback<PointerEvent>` | Reports the swipe once it covers the distance. |
| `onpointerup`, `onpointercancel` | `Callback<PointerEvent>` | End the press. |

`Swipe` is `Copy`. `edge_swipe_sx()` is `touch-action: pan-y pinch-zoom`: the
page still scrolls and zooms, a sideways move reaches the hook. It stops the
browser's own sideways pan of that box (inner scrollers keep theirs but from the
band), so spread
it on the main column, not on a horizontal scroller. The edge swipe
reads the direction from `use_direction` and the viewport width at each press.

## Accessibility

### Libero handles

- A tap, a scroll and a pinch keep working: the hooks prevent no default but an
  edge swipe's sideways pan of an inner scroller, and a second finger drops the
  press.
- The edge swipe starts past Android's system back zone, so Back from the
  screen edge still works.
- A mouse is ignored, so a desktop click or text selection never turns into a
  swipe.

### You must

- Offer the same action without a gesture: a swipe is a path-based gesture
  (WCAG 2.5.1, 2.5.7). The docs keep the burger as the single-pointer way to
  open the navigation; the demo has buttons.
- Announce what the swipe did with a live region, or move focus where it leads.

### Limits

- Touch and pen only. On iOS and in a mobile browser tab the system or the
  browser owns the very screen edge; the edge swipe band starts past it, but its
  inset was measured on Android only.
- Blitz has no touch input, so the hooks are inert in native windows.
