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
            sx: sx().width("160px").height("160px").background("grey.1"),
        }
    }
}
```

`fallback_src` only shows once `src` has actually failed to load - the component
watches the `<img>`'s `onerror` and swaps the source then, so a working `src`
never fetches the fallback.

`zoomable` wraps the image in a click-to-zoom overlay. The inline image becomes a
`<button>`, and clicking it opens the picture in a [Modal](modal.md) at up to
90vw/90vh, `object-fit: contain`. `zoomed_src` supplies a larger source for the
overlay when the inline one is a thumbnail.

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

A plain `Image` is an `<img>` carrying your `alt`. Leave `alt` empty for a purely
decorative image and it also gets `role="presentation"`, so a screen reader skips
it instead of announcing a filename.

A `zoomable` image is a real `<button>` with `type="button"` and
`aria-pressed` tracking the zoom state, so Tab reaches it and Space or Enter
zooms. Its accessible name is derived from `alt`: `Zoom in: <alt>` closed,
`Zoom out: <alt>` open, falling back to plain `Zoom in`/`Zoom out` when `alt` is
empty. The inner `<img>` is then `alt=""`/`role="presentation"`, since the button
already carries the name.

The overlay is a `Modal` + `Dialog`, so Escape closes it and focus is trapped
inside while it is open; the overlay itself is a button with `data-autofocus`, so
focus lands there on open and returns to the inline image on close.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The image source. |
| `fallback_src` | `String` | - | Shown in place of `src` once it fails to load. |
| `zoomed_src` | `String` | follows `src` | Source shown in the zoom overlay, if different from the inline image. |
| `fit` | `ImageFit` | `cover` | Maps onto `object-fit`. |
| `radius` | `ThemeAwareValue` | `0` | Corner radius - the radius scale, or any CSS length. |
| `alt` | `String` | required | Alt text. Empty marks the image decorative. |
| `zoomable` | `bool` | `false` | Wraps the image in a click-to-zoom overlay. |

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
