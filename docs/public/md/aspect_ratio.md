# AspectRatio

Crate: `libero`
Import: `use libero::components::AspectRatio;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/aspect_ratio.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Enforces a width-to-height ratio on its child, cropping it to fill the box.

Enforces a width-to-height ratio on its child, cropping it to fill the box.
`ratio` is a plain `f32` - write it as the division it reads as - and defaults
to the theme's `1.0`. The box has no size of its own, so a ratio only shows
once something gives it a width.

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
                sx: sx().background("primary").color("white"),
                align: "center",
                justify: "center",
                "The child fills the box"
            }
        }
    }
}
```

The child is stretched to `100%` in both axes and anything overflowing is
clipped (`overflow: hidden`), so an image or a video fills the frame rather than
setting its own height.

## Accessibility

Because it crops, make sure nothing meaningful lives at the edges of the child;
an image whose subject is cut off still needs `alt` text describing what the
reader can see.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `ratio` | `f32` | `1.0` | Width-to-height ratio, e.g. `16.0 / 9.0`. |
| `children` | `Element` | required | The child to crop, filling the box. |

Like every component, `AspectRatio` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`AspectRatioDefaults` on the theme - one field, the ratio used when the prop is
omitted.

| Field | Type | Description |
|---|---|---|
| `ratio` | `f32` | Default width-to-height ratio. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-aspect-ratio` | The default `aspect-ratio` for every `AspectRatio`. |

The `ratio` prop writes `--lsx-aspect-ratio-override` into the element's `style`
attribute, so a per-instance ratio never mints a new class.

## Data attributes

`AspectRatio` sets no state tokens of its own; a `states` prop is passed through
unchanged.
