# ImageCropper

Crate: `libero`
Import: `use libero::components::ImageCropper;`
Feature: `image-cropper` (off by default, turns on `slider`, `action-icon`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/image_cropper>
Index: [index.md](index.md) lists every other page
Description: A box with handles over an image, picking the part to keep by drag or arrow keys, free or at a fixed aspect, with a rect or circle mask.

A box with handles over an image, picking the part to keep. Drag the box, a
handle or the image around the box, pinch with two fingers, or use the arrow
keys on the box and its corners. `value` is a `CropRect` in fractions of the
image, so it fits any resolution; `to_pixels` turns it into pixels. It only
picks the box. To cut the picture, give a `FileField` a `crop`: a picked image
opens in a cropper first, and the field takes the cut file.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{CropRect, ImageCropper};

#[component]
fn Demo() -> Element {
    let mut crop = use_signal(|| None::<CropRect>);

    rsx! {
        ImageCropper {
            src: SAMPLE_IMAGE,
            alt: "A sample landscape",
            value: crop(),
            onchange: move |rect| crop.set(Some(rect)),
        }
    }
}
#
# const SAMPLE_IMAGE: &str = "/sample_image.png";
```

## Props

### `ImageCropper`

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The image: any URL, a `data:` URL included. The browser fetches it with a plain GET and the page's cookies, so a URL built from user data can act as a tracking beacon. A `javascript:` or `vbscript:` URL loads nothing and warns in debug builds. |
| `alt` | `String` | required | Describes the image. |
| `value` | `Option<CropRect>` | - | The box, in fractions of the image. Pair it with `onchange`. Unset starts centred at 80% of the largest box `aspect` allows, so it can move at once, and reports it once the image has loaded. A new `src` or `aspect` starts an unset box over. A set `value` stays as it is, so reset it yourself. |
| `onchange` | `EventHandler<CropRect>` | - | Fires on every move of the box, by a drag, a key or a button under the image. Without it the cropper only shows, with no buttons. |
| `aspect` | `f64` | - | Locks width over height, in image pixels: `1.0` is square, `16.0 / 9.0` wide. Unset is free. |
| `shape` | `CropShape` | `Rect` | `Circle` masks outside an ellipse, for an avatar. The rect is the same either way. |
| `min_size` | `f64` | `0.05` | The smallest side, a fraction of the image's. |
| `pan` | `bool` | `false` | Holds the box still, centred, and moves the image under it, as a phone's profile picture cropper does: a drag pans the image, a pinch, the wheel or the + and - keys zoom it. No resize handles; `value` stays the crop in fractions of the image. With an `aspect` on a picture of another shape the stage takes the box's shape, the picture centred on it as on a square, and `overhang` counts from that stage, so a wide picture zooms out until its width fits the box. |
| `controls` | `ImageCropperControls` | `Shown` | The buttons under the image, and with `pan` the zoom bar. `HiddenUntilFocus` (what `false` converts to) hides them from sight only: they show again while focus is inside them, so a keyboard and a screen reader keep them. `None` removes them; the box's keys stay. |
| `overhang` | `f64` | `0.0` | How far the box may reach past each side of the picture, a fraction of its size, 0 to 1: room to centre a logo or zoom out on a square. The cropper reserves that room round the picture and draws `backdrop` in it; `value` leaves 0 to 1 by as much. |
| `backdrop` | `Backdrop` | `Checkerboard` | What shows past the picture's edge, and behind its see-through pixels: a `Checkerboard` or a flat `Color` (`Backdrop::rgb(255, 255, 255)`). A flat colour is also what the cut paints there; a checkerboard leaves it transparent. |
| `disabled` | `bool` | `false` | Dims the cropper and takes no input. |
| `aria_label` | `String` | - | Names the box; the localization's `image_cropper.label` ("Crop area") when unset. |
| `onerror` | `EventHandler<()>` | - | Fires when `src` fails to load. The box is not drawn until `src` changes, so the alt text shows. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, `hidden_from`, `visible_from`, and any extra HTML attributes.

### `ImageCropPreview`

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The picture the cropper shows, as its `src`. |
| `alt` | `String` | required | Describes the crop. |
| `value` | `CropRect` | required | The crop, as the cropper's `value`; live when both read one signal. Past the picture's edge shows `backdrop`. |
| `shape` | `CropShape` | `Rect` | `Circle` dims the corners past the circle, so what sticks out of an avatar shows. |
| `aspect` | `f64` | `1.0` | The crop's width over its height in pixels, as the cropper's `aspect`. |
| `size` | `f64` | `96.0` | The longer side in pixels. |
| `backdrop` | `Backdrop` | `Checkerboard` | What shows past the picture's edge. |

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
| `overhang` | `f64` | `0.0` | As `overhang` above. |
| `backdrop` | `Backdrop` | `Checkerboard` | As `backdrop` above; also what fills the cut past the picture. |

### `CropEvent`

| Field | Type | Description |
|---|---|---|
| `rect` | `CropRect` | The box the `FileField` crop dialog applied. |
| `original` | `FileData` | The file as picked, before the cut. |

### `libero::platform::image`

| Prop | Type | Description |
|---|---|---|
| `file_from_bytes(name, content_type, bytes)` | `Option<FileData>` | The bytes as a file a form can post. `None` where no page holds files: a server render, Blitz. |
| `crop_image(file, rect, options).await` | `Option<FileData>` | The file cut to a `CropRect`, past the picture's edge too, as `CutOptions` say. `None` where no page can cut it (a server render, so call it from an event) or the picture would not decode. |
| `on_square(file, margin, options).await` | `Option<FileData>` | The picture centred on a square canvas, `margin` of its longer side free on every side, cut as `crop_image` cuts. |
| `has_transparency(source).await` | `Option<bool>` | Whether any pixel is not fully opaque, of a `&FileData` or a URL (`ImageSource`). A JPEG, or a PNG or lossy WebP without alpha, answers from its header; otherwise it reads the pixels and stops at the first see-through one. `None` when the URL would not load or the picture would not decode. |
| `image_size(source).await` | `Option<(u32, u32)>` | The width and height in pixels of a `&FileData` or a URL, turned as its EXIF says. |

### `CutOptions`

| Field | Type | Default | Description |
|---|---|---|---|
| `max_size` | `Option<u32>` | - | Scales the cut down so its longer side is at most this many pixels. |
| `backdrop` | `Backdrop` | `Checkerboard` | What fills past the picture's edge and, with a `Circle`, past the circle. A checkerboard leaves it transparent; a JPEG gets white. |
| `shape` | `CropShape` | `Rect` | `Circle` fills everything outside the ellipse inscribed in the cut with `backdrop`: a round avatar. |
| `output` | `ImageType` | `Keep` | The type written: `Keep` keeps a PNG, JPEG or WebP's and makes anything else a PNG; `Png`, `Jpeg` or `Webp` picks one, so a JPEG cut past its edge can stay see-through as a PNG. |

### `ImageSource`

| From | Variant | Description |
|---|---|---|
| `&FileData` | `ImageSource::File` | A picked or built file. |
| `&str, &String` | `ImageSource::Url` | Loaded with the page's `fetch` on the web and in a WebView, so a cross-origin URL needs CORS; Blitz decodes `data:` itself and loads `http(s)` through its network provider. `javascript:`, `file:` and the like load nothing. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ImageCropperPart::Image` | `image` | The image. |
| `ImageCropperPart::Backdrop` | `backdrop` | The checkerboard or colour behind the picture and its overhang; drawn with an `overhang` or a flat colour. |
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
| `Arrow` | On the box: moves it, with `pan` the image, as a drag does. On a corner: moves that corner, resizing the box. |
| `Shift+Arrow` | Ten times as far. |
| `Home`, `End` | On the box: moves it to the left or right edge, with `pan` the image. On a corner: moves its left or right side as far left or right as it goes. |
| `+`, `=`, `-` | With `pan`: + or = zooms the image in, - zooms it out, around the box's centre. |

### Libero handles

- The box is a `slider` tab stop named by `aria_label`, its value spoken as
  "50% by 50%, at 25%, 25%".
- The four corners are tab stops of their own, sliders named "Top left corner"
  and so on.
- Each slider's number rises with ArrowRight: the box's left edge, with `pan`
  the image's; a left corner's left edge, a right corner's width.
- Each handle is a 24px target around a 12px square, 44px on a touch screen.
- A drag focuses the box or the corner it grabbed, so the keys carry on from
  there.
- On a touch screen a finger on the image, on the box or beside it, drags the
  box without scrolling the page; a disabled cropper lets the page scroll. Two
  fingers pinch the box larger or smaller around its centre; so do Ctrl or
  Cmd with the mouse wheel and a trackpad pinch over the image. A plain wheel
  scrolls the page.
- With `pan` the whole cropper takes touches: one finger pans the image, two
  zoom it around their midpoint. The mouse wheel and a trackpad pinch zoom it
  around the pointer. Its value adds the zoom: "40% by 80%, at 30%, 10%, zoom
  100%".
- With `pan` a slider named "Zoom" under the image zooms around the box's
  centre, by one pointer or its keys, so no pinch is needed (WCAG 2.5.1). Every
  step zooms by the same factor. Its value is the zoom, "100%", the same number
  the box says. Minus and plus glyphs, hidden from assistive tech, mark its ends;
  a press on one zooms a step, as - and + do, and leaves focus where it is. One
  that would change nothing dims.
- Under the image a group named "Crop area controls" holds buttons for every drag
  (WCAG 2.5.7): "Move left", "Move up", "Move down" and "Move right" move the
  box 5% of the image, with `pan` the image under it; "Smaller" and "Larger"
  scale the box around its centre, without `pan`. A button whose press would
  leave the box as it is, such as "Move left" at the left edge, is
  `aria-disabled` and stays focusable. With `controls: false` the
  buttons and the zoom bar stay out of sight until focus reaches them; with
  `ImageCropperControls::None` they are gone and only the keys remain.
- With an `aspect`, a corner key resizes both sides together.
- The box and the corners describe their keys.

### You must

- Describe the image with `alt`.
- Translate the corner names and the spoken value with the localization.

### Example

An avatar crop, `ImageCropper { alt: "Your photo", aria_label: "Crop area",
aspect: 1.0, .. }`: Tab reaches the crop box, the arrows move it, and a screen
reader hears where it sits, such as "50% by 50%, at 25%, 25%".

### Limits

- The edge handles are pointer-only: the corners reach every size.
- A screen reader hears where the box is, not what it shows.
- Under Blitz a `FileField` with `crop` passes an AVIF through uncropped, with
  no dialog.
- Under Blitz a cut WebP is lossless, so it can be larger than the browser's
  lossy one.
- The overhang is room round the picture, so it takes layout space the picture's
  own size does not: a picture fills the container less by that much.
- Under Blitz the `platform::image` helpers read PNG, JPEG, WebP and BMP only:
  an SVG gives `None`.
- Under Blitz an image takes no `aspect-ratio`, so `pan` with an `aspect` keeps
  the picture's own stage there.

## Localization

`Localization.image_cropper` holds the box's name (`label`), its key hint
(`keys`), the corner names, and the crop dialog's `title`, `apply` and
`cancel`. `value` is the spoken box, with the holes `{width}`, `{height}`, `{x}`
and `{y}` in percent: "{width}% by {height}%, at {x}%, {y}%" in English. With
`pan`, `pan_keys` and `pan_value` take their place; `pan_value` adds `{zoom}`, in
percent of the starting zoom: "{width}% by {height}%, at {x}%, {y}%, zoom {zoom}%".
The pan mode's zoom slider is named by `zoom` ("Zoom") and speaks `zoom_value`
("{zoom}%"). The group of buttons under the image is named by `controls`, the
buttons by `move_up`, `move_down`,
`move_left`, `move_right`, `smaller` and `larger`. A crop dialog that cannot load
the image says `load_failed`, one whose cut fails `crop_failed`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-image-cropper-x` / `--lsx-image-cropper-y` | The box's top left corner, a fraction of the image. |
| `--lsx-image-cropper-width` / `--lsx-image-cropper-height` | The box's size, a fraction of the image. |

With `pan` the four above place the still box in fractions of the cropper, and
`--lsx-image-cropper-scale`, `--lsx-image-cropper-image-x` and
`--lsx-image-cropper-image-y` scale and offset the image under it. With an
`overhang` or a flat `backdrop`, `--lsx-image-cropper-over` is the overhang and
`--lsx-image-cropper-span` the picture and its overhang over the picture.

## Data attributes

`data-state` on the root carries `circle`, `pan`, `hidden-controls`, `backdrop` and `disabled` when they apply. Each
handle is `data-slot="handle"` with `data-grip` one of `n`, `s`, `e`, `w`,
`ne`, `nw`, `se`, `sw`.
