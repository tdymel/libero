# Lightbox

Crate: `libero`
Import: `use libero::hooks::{LightboxItem, LightboxOptions, use_lightbox};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_lightbox.rs>
Index: [index.md](index.md) lists every other page
Description: A modal image viewer, `use_modal` with a gallery around it, with zoom, pan, captions and a thumbnail strip.

A modal image viewer. `use_lightbox` is [`use_modal`](modal.md) with a gallery
around it. Each opening carries its pictures and where to start, and focus
returns to the thumbnail that opened it. `Image { zoomable }` ([Image](image.md))
is this viewer with one picture.

Double-click or press `z` to step through 2x, 4x and 8x and back to fitted. The
zoom buttons, `+` and `-` zoom in finer steps. Scroll to zoom at the cursor.
Drag, click or use the arrows to pan. At the edge of a pan the arrows move to
the next picture, so the keyboard never gets stuck.

## Usage

The options are shared by every opening and read once. The gallery travels with
each opening, so it can change between openings.

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

## Accessibility

Give every picture its own `alt`. With `zoom` on, the picture showing is a tab
stop that takes the zoom and pan keys. Its description lists them, and a status
message reads each new zoom level. In the thumbnail strip, the arrows, Home and
End move along the strip and change the picture with it. Esc closes the viewer.

## API

### `use_lightbox`

```rust,ignore
pub fn use_lightbox(options: LightboxOptions) -> ModalHandle<LightboxOpening>
```

Returns the same `ModalHandle` as `use_modal`, and it and `Opening` behave the
same. See [Modal](modal.md). `open_with` takes a `LightboxItem`, a
`Vec<LightboxItem>` (starting at the first) or a `(Vec<LightboxItem>, usize)`
pair. An index past the end shows the last picture.

### `LightboxOptions`

`LightboxOptions` implements `Default`, so a literal sets only what it needs,
`LightboxOptions { thumbnails: false, ..LightboxOptions::default() }`.

| Field | Type | Default | Description |
|---|---|---|---|
| `zoom` | `bool` | `true` | Lets the zoom buttons, the wheel, a double-click, `z`, `+` and `-` zoom, and a drag, a click or the arrows pan. On desktop and mobile a drag stops once the pointer leaves the picture. |
| `max_zoom` | `Option<f64>` | `8.0` | Upper scale bound. Unset, the theme's. |
| `thumbnails` | `bool` | `true` | The strip under the stage. Never shown for one picture. |
| `captions` | `bool` | `true` | Shows each item's caption. |
| `controls` | `bool` | `true` | The previous and next arrows. |
| `preload` | `usize` | `1` | Pictures on each side loaded at once. The rest load lazily. |
| `close_on_swipe_down` | `bool` | `true` | A downward touch swipe closes. Off while zoomed. |
| `aria_label` | `Option<String>` | `"Gallery"` | Names the dialog. Unset, the localization's label. |

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
| `width` | `&'static str` | The dialog's widest extent. Below the `sm` breakpoint, or under `30rem` tall, the viewer is full screen instead. |
| `stage_height` | `&'static str` | Every frame's height. A larger picture is scaled down into it, and a smaller one is never scaled up. |
| `thumbnail_size` | `&'static str` | One thumbnail's edge, capping the strip's width. |
| `thumbnails_per_view` | `f64` | Thumbnails visible at once before the strip scrolls. |
| `thumbnails_gap` | `Size` | Space between thumbnails. |
| `max_zoom` | `f64` | Default upper scale bound, `1.0` being the picture as first shown. |

The words are `LightboxLabels` in the [localization](localization.md). `label`
is the dialog's default name, `thumbnails` the strip's region name, `thumbnail`
a thumbnail's name (`{n}` is the slide number), `keys` the zoomable picture's
description, `zoomed` the zoom announcement (`{n}` is the scale in percent) and
`fitted` the announcement once back to fitted, and `zoom_in` and `zoom_out`
name the zoom buttons. The close button's name is `common.close`.

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
