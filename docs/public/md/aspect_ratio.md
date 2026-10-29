# AspectRatio

Crate: `libero`
Import: `use libero::components::AspectRatio;`
Feature: `aspect-ratio` (on by default, turns on `box`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/aspect_ratio.rs>
Index: [index.md](index.md) lists every other page
Description: Holds its child to a width-to-height ratio, filling the box.

Holds its child to a width-to-height ratio, filling the box. Write `ratio` as a
division, like `16.0 / 9.0`. The box has no size of its own, so give it a
width. An image or a video is cropped with `object-fit: cover`, not stretched,
and an `a`, `picture` or `span` child becomes a block. Give any other inline
child `display: block`, and a `picture`'s inner `img` the full size. The size
and `object-fit` reach the direct child only: an image inside a wrapper is not
cropped unless it gets `width: 100%`, `height: 100%` and `object-fit: cover`
itself. A text child that needs more room grows the box past its ratio instead
of being cut off.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::{AspectRatio, Flex}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        AspectRatio {
            sx: sx().width("240px"),
            Flex {
                sx: sx().background("primary").color("primary-contrast"),
                align: "center",
                justify: "center",
                "The child fills the box"
            }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `ratio` | `f32` | `1.0` | Width-to-height ratio, e.g. `16.0 / 9.0`. |
| `children` | `Element` | required | The child that fills the box; an image or a video is cropped to it. An `a`, `picture` or `span` child becomes a block; give any other inline child `display: block`, and a `picture`'s inner `img` the full size. The size and `object-fit` reach the direct child only: an image inside a wrapper needs `width: 100%`, `height: 100%` and `object-fit: cover` itself. A text child is never clipped: when it needs more room, at larger text or a narrow width, it grows the box taller than its ratio. |

Like every component, `AspectRatio` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- A focused child keeps its focus ring: it is drawn inside the child's edges.
- A text child grows the box past its ratio at larger text, so nothing is cut off.

### You must

- Keep nothing meaningful at an image's or a video's edges: they get cropped.
- Leave room below a text child: at larger text it grows the box taller than its ratio.
- Give an image `alt` text that describes what the reader can see.

### Example

A video thumbnail in `AspectRatio { ratio: 16.0 / 9.0, .. }` with an image
whose `alt` is "Sunset over the harbour": the picture is cropped at its edges,
and the play button in its centre keeps its focus ring.

## Theme defaults

`AspectRatioDefaults` on the theme holds the ratio used when the prop is omitted.

| Field | Type | Description |
|---|---|---|
| `ratio` | `f32` | Default width-to-height ratio. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-aspect-ratio` | The default `aspect-ratio` for every `AspectRatio`. |

## Data attributes

`AspectRatio` sets no state tokens of its own. A `states` prop passes through
unchanged.
