# Image

Crate: `libero`
Import: `use libero::components::Image;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/image.rs>
Index: [index.md](index.md) lists every other page
Description: An `<img>` with a fallback source on load error, rounded corners and an optional click-to-zoom overlay.

An `<img>` with a fallback source on load error, optional rounded corners, and
an optional click-to-zoom overlay. `fit` maps straight onto `object-fit`;
`radius` takes a step on the radius scale or any CSS.

## Usage

`fallback_src` loads only once `src` fails, so a working `src` never fetches it.

```rust
use dioxus::prelude::*;
use libero::{components::Image, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Image {
            src: "/landscape.png",
            fallback_src: "/placeholder.png",
            alt: "A stylised landscape",
            sx: sx().width("160px").height("160px").background("muted.1"),
        }
    }
}
```

`zoomable` makes the image a button that opens it in a single-picture
[Lightbox](lightbox.md), where it can be zoomed and panned. `zoomed_src` gives
the overlay a larger source when the inline one is a thumbnail. Inside a linked
[ImageItem](image_list.md) the link wins and `zoomable` is ignored, with a
warning in a debug build.

```rust
use dioxus::prelude::*;
use libero::{components::Image, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Image {
            src: "/landscape-thumb.png",
            zoomed_src: "/landscape-full.png",
            alt: "A stylised landscape",
            radius: "md",
            zoomable: true,
            sx: sx().width("160px").height("160px"),
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The image source. |
| `fallback_src` | `Option<String>` | `None` | Shown in place of `src` once it fails to load. |
| `zoomed_src` | `Option<String>` | `None` | A larger source for the zoom overlay. Falls back to `src`. |
| `fit` | `ImageFit` | `cover` | Maps onto `object-fit`. |
| `radius` | `ThemeAwareValue` | `0` | Corner radius, a step on the radius scale, or any CSS, e.g. `radius: "0"`. |
| `alt` | `Option<String>` | `None` | What the picture shows. A debug build warns when neither `alt` nor `decorative` is set. |
| `decorative` | `bool` | `false` | Marks the picture as decoration, hidden from screen readers. Wins over `alt`, with a warning in a debug build. |
| `zoomable` | `bool` | `false` | Opens the picture in a single-picture Lightbox on click. Ignored inside a linked `ImageItem`, with a warning. The zoom button then takes `class`, `states` and the extra attributes, so `aria_label` or `data-*` land on it; `loading`, `decoding`, `fetchpriority`, `srcset`, `sizes`, `crossorigin`, `referrerpolicy` and `usemap` stay on the `<img>`. |
| `loading` | `ImageLoading` | `eager` | The `<img>`'s `loading`. `lazy` loads the picture only when it nears the viewport. |
| `parts` | `Parts<ImagePart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

`ImageFit` takes `fill`, `contain`, `cover`, `none` or `scale-down`.
`ImageLoading` takes `eager` or `lazy`.

Like every component, `Image` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

With `zoomable` only: `sx` then styles the zoom button, and the picture is this part. Without it the `<img>` is the root.

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ImagePart::Image` | `image` | The `<img>` inside the zoom button. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Enter` or `Space` | On a zoomable image: opens the overlay. |
| `Escape` | Closes the overlay, as do the backdrop and the Close button. |

### Libero handles

- `decorative` renders `alt=""` and `role="presentation"`.
- A zoomable image is a button named after its `alt`, "Zoom in: <alt>" (the
  localization's `image.zoom_named`).
- An image with neither `alt` nor `decorative` warns in a debug build and
  renders no `alt`, so a checker still flags it.
- A zoomable image with no `alt`, an empty one or `decorative` warns in a debug
  build: a checker passes its bare "Zoom in" button and "Gallery" dialog.

### You must

- Give every image an `alt`, or set `decorative` for one that carries nothing.

### Example

A product photo, `Image { alt: "Blue running shoe, side view", zoomable: true,
.. }`: a screen reader reads the button "Zoom in: Blue running shoe, side
view", and Escape closes the zoomed view.

## Theme defaults

`ImageDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `fit` | `ImageFit` | Default `fit` when the prop is omitted. |
| `radius` | `&'static str` | CSS length used when an `Image` sets no `radius` of its own. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-image-radius` | The theme's default corner radius. |
| `--lsx-image-radius-override` | Set from the `radius` prop through the radius scale. Wins over the theme value. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `fit-fill` / `fit-contain` / `fit-cover` / `fit-none` / `fit-scale-down` | The `fit` in effect, on the `<img>`. That is the root unless the image is `zoomable`. |
| `zoomed` | On a `zoomable` image's `<button>` root, while the overlay is open. |
