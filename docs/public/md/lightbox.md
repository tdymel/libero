# Lightbox

Crate: `libero`
Import: `use libero::hooks::{LightboxItem, LightboxOptions, use_lightbox};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_lightbox.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A modal image viewer - `use_modal` with a gallery around it: zoom, pan, captions and a thumbnail strip.

A modal image viewer. `use_lightbox` is [`use_modal`](modal.md) with a gallery
around it: each opening carries its pictures and where to start, and focus goes
back to the thumbnail that opened it. `Image { zoomable }` ([Image](image.md))
is this viewer with one picture.

Double-click or press `z` to zoom, scroll to zoom about the cursor, drag or use
the arrows to pan. At the edge of a pan the arrows move to the next picture, so
a zoomed picture never traps the keyboard.

## Usage

The options are what every opening shares and are read once, when the hook is
first called. The gallery travels with each opening, so it can change between
openings.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Image},
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
};

#[derive(Clone, PartialEq)]
struct Photo {
    src: String,
    alt: String,
    title: String,
}

#[component]
fn Demo(photos: Vec<Photo>) -> Element {
    let lightbox = use_lightbox(LightboxOptions::default());
    let items: Vec<LightboxItem> = photos
        .iter()
        .map(|p| LightboxItem::new(&p.src, &p.alt).caption(&p.title))
        .collect();

    rsx! {
        for (index, photo) in photos.iter().enumerate() {
            Box {
                component: "button",
                r#type: "button",
                aria_label: "Open {photo.alt}",
                onclick: {
                    let items = items.clone();
                    move |_| { lightbox.open_with((items.clone(), index)); }
                },
                Image { src: "{photo.src}", alt: "", fit: "cover" }
            }
        }
    }
}
```

`open_with` takes a `LightboxItem`, a `Vec<LightboxItem>` (starting at the
first), or a `(Vec<LightboxItem>, usize)` pair. An index past the end shows the
last picture.

## Zoom and pan

| Input | Action |
|---|---|
| Double-click, `z` | Toggle between fitted and `min(2, max_zoom)`; a double-click zooms about the cursor |
| Wheel | Zoom about the cursor, between fitted and `max_zoom` |
| Drag | Pan while zoomed |
| `ArrowLeft` / `ArrowRight` | Pan while zoomed; at the pan edge, or when fitted, the previous / next picture |
| `ArrowUp` / `ArrowDown` | Pan while zoomed |
| `Home` / `End` | First / last picture |
| Swipe down (touch) | Close, when `close_on_swipe_down` and not zoomed |
| `Escape`, backdrop, **Close** | Close |

The keys act while the picture has focus. A pan is clamped so the picture
always covers its frame; moving to another picture resets the zoom. With focus
on the carousel's track the arrows page through the pictures as in any
[Carousel](carousel.md).

Pan tracking outside the picture needs pointer capture, which Blitz and the
WebView floor do not offer: there, a drag stops following once the pointer
leaves the picture, while the wheel, the keys and double-click still work.

## What it is made of

A [Dialog](dialog.md) in the hook's modal, with the theme's `"Close"` button;
a [Carousel](carousel.md) of fixed-height frames, one per picture, with the
controls and no indicators; the caption as a `<p>`; and a second `Carousel` of
thumbnail buttons. One picture renders the frame alone, with no carousel and no
strip. The lightbox dims through the shared `OverlayDefaults`, like every other
modal.

## Accessibility

Name the dialog with `aria_label`; it falls back to the theme's `"Gallery"`.
Give every picture its own `alt`.

The picture showing is a tab stop (with `zoom` on) and takes the zoom and pan
keys. In the thumbnail strip the arrows, `Home` and `End` move along the strip
and change the picture with it.

## API

### `use_lightbox`

```rust,ignore
pub fn use_lightbox(options: LightboxOptions) -> ModalHandle<LightboxOpening>
```

Returns the same `ModalHandle` as `use_modal`; every method on it and on
`Opening` behaves identically. See [Modal](modal.md).

### `LightboxOptions`

`Default`, so a literal overrides only what it needs:
`LightboxOptions { thumbnails: false, ..LightboxOptions::default() }`.

| Field | Type | Default | Description |
|---|---|---|---|
| `zoom` | `bool` | `true` | Wheel, double-click and `z` zoom; drag and arrow pan. |
| `max_zoom` | `Option<f64>` | theme (`3.0`) | Upper scale bound. |
| `thumbnails` | `bool` | `true` | The strip under the stage. Never shown for one picture. |
| `captions` | `bool` | `true` | Shows each item's caption, linked to its picture. |
| `controls` | `bool` | `true` | The previous / next arrows. |
| `preload` | `usize` | `1` | Neighbours each side loaded `eager`; the rest are `lazy`. |
| `close_on_swipe_down` | `bool` | `true` | A downward touch swipe closes. Off while zoomed. |
| `aria_label` | `Option<String>` | theme (`"Gallery"`) | Names the dialog. |

### `LightboxItem`

`LightboxItem::new(src, alt)`, then `.caption(..)` and `.thumbnail(..)`.

| Field | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The picture. |
| `thumbnail_src` | `Option<String>` | follows `src` | A smaller source for the strip. |
| `alt` | `String` | required | The picture's text alternative. |
| `caption` | `Option<String>` | - | Shown under the stage. |

### `LightboxOpening`

| Field | Type | Description |
|---|---|---|
| `items` | `Vec<LightboxItem>` | The gallery. |
| `index` | `usize` | The picture to start on. |

## Theme defaults

`LightboxDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `width` | `&'static str` | The dialog's widest extent. Below the `sm` breakpoint, or under `30rem` tall (a phone on its side), the viewer is full screen instead, inside the safe-area insets. |
| `stage_height` | `&'static str` | Every frame's height; full screen, the room the close button, caption and thumbnails leave. A larger picture is scaled down into it; a smaller one shows at its natural size, never upscaled. |
| `thumbnail_size` | `&'static str` | One thumbnail's edge, capping the strip's width. |
| `thumbnails_per_view` | `f64` | Thumbnails visible at once before the strip scrolls. |
| `thumbnails_gap` | `Size` | Space between thumbnails. |
| `max_zoom` | `f64` | Default upper scale bound, `1.0` being the picture as first shown. |
| `label` | `&'static str` | The dialog's default name. |
| `close_label` | `&'static str` | The close button's name. |
| `thumbnails_label` | `&'static str` | The strip's region name. |
| `thumbnail_label` | `&'static str` | A thumbnail's name; `{n}` is the slide number. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-lightbox-width` | The dialog's widest extent. |
| `--lsx-lightbox-stage-height` | Every frame's height. |
| `--lsx-lightbox-thumbnail-size` | One thumbnail's edge. |
| `--lsx-lightbox-thumbnails-gap` | Space between thumbnails. |

## Data attributes

State tokens on the pictures' and thumbnails' `data-state`, space separated.

| Token | Condition |
|---|---|
| `zoomable` | On a picture that zooms and is fitted. |
| `swipe` | On a fitted picture while a swipe down closes. |
| `zoomed` | On the picture showing, while zoomed. |
| `dragging` | On the picture showing, during a pan or swipe. |
| `current` | On the current thumbnail. |

Each frame carries `data-lightbox-frame="<index>"`.
