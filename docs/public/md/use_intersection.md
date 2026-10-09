# Intersection

Crate: `libero`
Import: `use libero::hooks::{InViewport, Intersection, IntersectionEntry, IntersectionOptions, use_in_viewport, use_intersection};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/intersection.rs>
Index: [index.md](index.md) lists every other page
Description: Reports how much of an element is visible, with a root, margin and thresholds; polled on Blitz, never intersecting in a server render.

`use_intersection(options) -> Intersection` watches one element with the
browser's `IntersectionObserver`, polled on Blitz.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, ScrollArea},
    hooks::{IntersectionOptions, use_intersection},
    sx::sx,
};

#[component]
fn Reveal() -> Element {
    let seen = use_intersection(IntersectionOptions {
        thresholds: vec![0.0, 0.5, 1.0],
        ..Default::default()
    });
    let percent = seen.entry.read().map_or(0, |entry| (entry.ratio * 100.0).round() as u32);

    // No `root`: the viewport's observer also clips by the scrolling area around the element.
    rsx! {
        Box { sx: sx().width("16rem").height("8rem").border("1px solid currentColor"),
            ScrollArea { aria_label: "Scrolling box",
                div { style: "height: 12rem;", "Scroll down" }
                div {
                    onmounted: move |event| seen.on_mounted.call(event),
                    ..seen.attributes,
                    "{percent}% visible"
                }
                div { style: "height: 12rem;" }
            }
        }
    }
}
```

## Wiring and options

Give `on_mounted` to the element's `onmounted`, spread its `attributes` on it (a
WebView finds the element by them) and read `entry`, a signal of
`Option<IntersectionEntry>` with `is_intersecting` and the visible `ratio`. The
options name a `root` element (the viewport by default), a `root_margin`, the
`thresholds` at which the entry updates, and `once`, which stops observing after
the first sighting.

## In the viewport

`use_in_viewport() -> InViewport` is the same with the defaults, with
`visible` as a bool. Where nothing can observe (a server render) `entry` stays
`None` and the bool is `false`.

## API

```rust,ignore
pub fn use_intersection(options: IntersectionOptions) -> Intersection
pub fn use_in_viewport() -> InViewport

pub struct IntersectionOptions {
    pub root: Option<ElementHandle>,
    pub root_margin: String,
    pub thresholds: Vec<f64>,
    pub once: bool,
}

pub struct Intersection {
    pub on_mounted: Callback<MountedEvent>,
    pub attributes: Vec<Attribute>,
    pub entry: ReadSignal<Option<IntersectionEntry>>,
}

pub struct InViewport {
    pub on_mounted: Callback<MountedEvent>,
    pub attributes: Vec<Attribute>,
    pub visible: ReadSignal<bool>,
}
```

| Option | Default | Description |
|---|---|---|
| `root` | `None` | The element whose box clips the target; the viewport when `None`. |
| `root_margin` | `"0px"` | CSS margin that grows or shrinks the root's box, like `"100px 0px"`. |
| `thresholds` | `[0.0]` | Visible ratios (0 to 1) at which the entry updates. |
| `once` | `false` | Stops observing once the element intersected; `entry` keeps that state. |

| Field | Type | Description |
|---|---|---|
| `IntersectionEntry::is_intersecting` | `bool` | Whether any part of the element is inside the root. |
| `IntersectionEntry::ratio` | `f64` | The visible share of the element's box, 0 to 1. |

`Intersection` and `InViewport` are `Clone`. Changing an option re-observes the element.

## Accessibility

### Libero handles

- The observer is dropped when the component unmounts, and replaced when an
  option changes, so one element never has two.

### You must

- Keep content that matters in the document and reachable by keyboard;
  `use_intersection` only reports, it hides nothing. An infinite list still
  needs a "Load more" button.
- Gate reveal animations on the reader's motion setting (`use_accessibility`).

### Example

An endless product list that loads the next page once its last row comes into
view: a "Load more" button under the list does the same for a keyboard user,
and every loaded row stays in the document.

### Limits

- The web and a WebView (desktop, Android) use an `IntersectionObserver`. A
  WebView finds the element by `attributes`, so spread them on it; without them
  nothing is observed there. A `root` needs `root.attributes()` spread on it
  the same way; without them a WebView observes against the viewport (a
  warning in debug builds). Blitz has no observer and is polled instead: at
  each render, after a scroll or input, and every 500 ms, so a change no scroll
  or input caused (a window resize, an animation) shows up to 500 ms late. In a
  server render `entry` stays `None`: treat `None` as "unknown" and show lazy
  content, rather than waiting for a sighting that never comes.
