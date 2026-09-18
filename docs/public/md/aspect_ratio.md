# AspectRatio

Crate: `libero`
Import: `use libero::components::AspectRatio;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/aspect_ratio.rs>
Index: [index.md](index.md) lists every other page
Description: Enforces a width-to-height ratio on its child, cropping it to fill the box.

Enforces a width-to-height ratio on its child, cropping it to fill the box.
Write `ratio` as a division, like `16.0 / 9.0`. The box has no size of its own,
so give it a width. The child stretches to fill it and the overflow is clipped,
which suits an image or a video.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::{AspectRatio, Flex}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        AspectRatio {
            ratio: 16.0 / 9.0,
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

## Accessibility

The edges of the child get cropped, so keep nothing meaningful there. An image
still needs `alt` text that describes what the reader can see.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `ratio` | `f32` | `1.0` | Width-to-height ratio, e.g. `16.0 / 9.0`. |
| `children` | `Element` | required | The child to crop, filling the box. |

Like every component, `AspectRatio` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

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
