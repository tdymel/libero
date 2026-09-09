# Image

Crate: `libero`
Import: `use libero::components::Image;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/image.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An img with a fallback source on load error, rounded corners, and an optional click-to-zoom overlay.

An `<img>` with a fallback source on load error, optional rounded corners, and
an optional click-to-zoom overlay. `fit` maps straight onto `object-fit`;
`radius` takes the radius scale or any CSS length.

## Usage

`Image` fills its box (`width: 100%; height: 100%`), so the box has to be sized
before `fit` means anything - that is what the `sx` below is for. `alt` is
required; an empty `alt` marks the image decorative and sets
`role="presentation"`.

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

`fallback_src` only shows once `src` has actually failed to load - the component
watches the `<img>`'s `onerror` and swaps the source then, so a working `src`
never fetches the fallback.

`zoomable` wraps the image in a click-to-zoom overlay. The inline image becomes a
`<button>`, and clicking it opens the picture in a single-picture
[Lightbox](lightbox.md): fitted to the stage with `object-fit: contain`, then
double-click, `z` or the wheel to zoom, and drag or the arrows to pan. No
thumbnails, captions or arrows. `zoomed_src` supplies a larger source for the
overlay when the inline one is a thumbnail. Inside a linked
[ImageItem](image_list.md) the link wins: `zoomable` is ignored, with a warning
in a debug build, since a `<button>` inside an `<a>` is invalid HTML.

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

## Accessibility

Leave `alt` empty only for a purely decorative image. A `zoomable` image takes
its names from `alt` - "Zoom in: <alt>" on the button, and the enlarged
picture's - so give it one.

A `zoomable` image is a tab stop: Space or Enter zooms, and Escape, the backdrop
or the dialog's **Close** button close the overlay again.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The image source. |
| `fallback_src` | `String` | - | Shown in place of `src` once it fails to load. |
| `zoomed_src` | `String` | follows `src` | Source shown in the zoom overlay, if different from the inline image. |
| `fit` | `ImageFit` | `cover` | Maps onto `object-fit`. |
| `radius` | `Size` | `0` | Corner radius, a step on the radius scale. Anything else goes through `sx`. |
| `alt` | `String` | required | Alt text. Empty marks the image decorative. |
| `zoomable` | `bool` | `false` | Wraps the image in a click-to-zoom overlay. Ignored inside a linked `ImageItem`. |

`ImageFit` takes `fill`, `contain`, `cover`, `none` or `scale-down`.

Like every component, `Image` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

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
| `--lsx-image-radius-override` | Set from the `radius` prop, resolved through the radius scale; wins over the theme value. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `fit-fill` / `fit-contain` / `fit-cover` / `fit-none` / `fit-scale-down` | The `fit` in effect. On the `<img>` - which is the root only when it is not `zoomable`. |
| `zoomed` | On a `zoomable` image's `<button>` root, while the overlay is open. |
