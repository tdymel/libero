# ImageCropper

Crate: `libero`
Import: `use libero::components::ImageCropper;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/image_cropper>
Index: [index.md](index.md) lists every other page
Description: A box with handles over an image, picking the part to keep by drag or arrow keys, free or at a fixed aspect, with a rect or circle mask.

A box with handles over an image, picking the part to keep. Drag the box, a
handle or the image around the box, pinch with two fingers, or use the arrow
keys on the box and its corners. `value` is a
`CropRect` in fractions of the image, so it fits any resolution; `to_pixels`
turns it into pixels.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{CropRect, ImageCropper};

#[component]
fn Demo() -> Element {
    let mut crop = use_signal(|| None::<CropRect>);

    rsx! {
        ImageCropper {
            src: "/photo.jpg",
            alt: "Holiday photo",
            aspect: 1.0,
            value: crop(),
            onchange: move |rect| crop.set(Some(rect)),
        }
    }
}
```

An avatar picker: a square box with a round mask, cropped on pick and scaled
to at most 512 px:

```rust,ignore
FileField {
    accept: "image/*",
    crop: CropOptions { aspect: Some(1.0), shape: CropShape::Circle, max_size: Some(512) },
    value: avatar(),
    onchange: move |files| avatar.set(files),
}
```

## Cutting the picture

It only picks the box. To cut the picture, give a `FileField` a `crop`: a
picked image opens in a cropper first, and the field takes the cut file.

## Props

### `ImageCropper`

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The image: any URL, a `data:` URL included. |
| `alt` | `String` | required | Describes the image. |
| `value` | `Option<CropRect>` | - | The box, in fractions of the image. Pair it with `onchange`. Unset starts centred at 80% of the largest box `aspect` allows, so it can move at once, and reports it once the image has loaded. A new `src` or `aspect` starts it over. |
| `onchange` | `EventHandler<CropRect>` | - | Fires on every move of the box, by a drag, a key or a button under the image. Without it the cropper only shows, with no buttons. |
| `aspect` | `f64` | - | Locks width over height, in image pixels: `1.0` is square, `16.0 / 9.0` wide. Unset is free. |
| `shape` | `CropShape` | `Rect` | `Circle` masks outside an ellipse, for an avatar. The rect is the same either way. |
| `min_size` | `f64` | `0.05` | The smallest side, a fraction of the image's. |
| `pan` | `bool` | `false` | Holds the box still, centred, and moves the image under it, as a phone's profile picture cropper does: a drag pans the image, a pinch, the wheel or the + and - keys zoom it. No resize handles; `value` stays the crop in fractions of the image. |
| `controls` | `bool` | `true` | `false` hides the buttons under the image, and with `pan` the zoom bar, from sight only: they show again while focus is inside them, so a keyboard and a screen reader keep them. The box's keys stay. |
| `disabled` | `bool` | `false` | Dims the cropper and takes no input. |
| `aria_label` | `String` | - | Names the box; the localization's `image_cropper.label` ("Crop area") when unset. |
| `onerror` | `EventHandler<()>` | - | Fires when `src` fails to load. The box is not drawn until `src` changes, so the alt text shows. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

### `CropRect`

| Field | Type | Description |
|---|---|---|
| `x, y` | `f64` | The top left corner, a fraction of the image's width and height. |
| `width, height` | `f64` | The size, a fraction of the image's. `CropRect::FULL` is the whole image. |
| `to_pixels(width, height)` | `PixelRect` | The box in pixels of an image that size, rounded. |

### `CropOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `aspect` | `Option<f64>` | - | As `aspect` above. |
| `shape` | `CropShape` | `Rect` | As `shape` above. |
| `pan` | `bool` | `false` | As `pan` above. |
| `max_size` | `Option<u32>` | - | Scales the crop down so its longer side is at most this many pixels. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ImageCropperPart::Image` | `image` | The image. |
| `ImageCropperPart::Mask` | `mask` | The dimmed image outside the box; a drag on it moves the box. |
| `ImageCropperPart::Box` | `box` | The crop box, a tab stop. |
| `ImageCropperPart::Frame` | `frame` | Over the box: takes its drags and holds the handles. |
| `ImageCropperPart::Handle` | `handle` | One of the eight resize handles. |
| `ImageCropperPart::Zoom` | `zoom` | With `pan`: the bar under the image holding the zoom slider. |
| `ImageCropperPart::Controls` | `controls` | The buttons under the image that move the box and, without `pan`, make it smaller or larger. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Arrow` | On the box: moves it. On a corner: moves that corner, resizing the box. |
| `Shift+Arrow` | Ten times as far. |
| `+`, `-` | With `pan`: zoom the image in or out around the box's centre. |

### Libero handles

- The box is a `slider` tab stop named by `aria_label`, its value spoken as
  "50% by 50%, at 25%, 25%".
- The four corners are tab stops of their own, sliders named "Top left corner"
  and so on.
- Each handle is a 24px target around a 12px square.
- A drag focuses the box or the corner it grabbed, so the keys carry on from
  there.
- On a touch screen a finger on the image, on the box or beside it, drags the
  box without scrolling the page; a disabled cropper lets the page scroll. Two
  fingers pinch the box larger or smaller around its centre; so do the mouse
  wheel and a trackpad pinch over the image.
- With `pan` the whole cropper takes touches: one finger pans the image, two
  zoom it around their midpoint. The mouse wheel and a trackpad pinch zoom it
  around the pointer. Its value adds the zoom: "40% by 80%, at 30%, 10%, zoom
  100%".
- With `pan` a slider named "Zoom" under the image zooms around the box's
  centre, by one pointer or its keys, so no pinch is needed (WCAG 2.5.1). Every
  step zooms by the same factor. Its value is the zoom, "100%".
- Under the image a group named by `aria_label` holds buttons for every drag
  (WCAG 2.5.7): "Move left", "Move up", "Move down" and "Move right" move the
  box 5% of the image, with `pan` the image under it; "Smaller" and "Larger"
  scale the box around its centre, without `pan`. With `controls: false` the
  buttons and the zoom bar stay out of sight until focus reaches them.
- With an `aspect`, a corner key resizes both sides together.
- The box and the corners describe their keys.

### You must

- Describe the image with `alt`.
- Translate the corner names and the spoken value with the localization.

### Limits

- The edge handles are pointer-only: the corners reach every size.
- A screen reader hears where the box is, not what it shows.
- Under Blitz a `FileField` with `crop` passes an AVIF through uncropped, with
  no dialog.
- Under Blitz a cut WebP is lossless, so it can be larger than the browser's
  lossy one.

## Localization

`Localization.image_cropper` holds the box's name (`label`), its key hint
(`keys`), the corner names, and the crop dialog's `title`, `apply` and
`cancel`. `value` is the spoken box, with the holes `{width}`, `{height}`, `{x}`
and `{y}` in percent: "{width}% by {height}%, at {x}%, {y}%" in English. With
`pan`, `pan_keys` and `pan_value` take their place; `pan_value` adds `{zoom}`, in
percent of the starting zoom: "{width}% by {height}%, at {x}%, {y}%, zoom {zoom}%".
The pan mode's zoom slider is named by `zoom` ("Zoom") and speaks `zoom_value`
("{zoom}%"). The buttons under the image are named by `move_up`, `move_down`,
`move_left`, `move_right`, `smaller` and `larger`. A crop dialog that cannot load
the image says `load_failed`, one whose cut fails `crop_failed`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-image-cropper-x` / `--lsx-image-cropper-y` | The box's top left corner, a fraction of the image. |
| `--lsx-image-cropper-width` / `--lsx-image-cropper-height` | The box's size, a fraction of the image. |

With `pan` the four above place the still box in fractions of the cropper, and
`--lsx-image-cropper-scale`, `--lsx-image-cropper-image-x` and
`--lsx-image-cropper-image-y` scale and offset the image under it.

## Data attributes

`data-state` on the root carries `circle`, `pan`, `hidden-controls` and `disabled` when they apply. Each
handle is `data-slot="handle"` with `data-grip` one of `n`, `s`, `e`, `w`,
`ne`, `nw`, `se`, `sw`.
