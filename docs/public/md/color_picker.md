# ColorPicker

Crate: `libero`
Import: `use libero::components::{ColorPicker, ColorCode};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/color>
Index: [index.md](index.md) lists every other page
Description: A saturation panel and a hue slider, with optional alpha and preset swatches, over one `ColorCode`. Also documents `HueSlider`, `AlphaSlider` and `ColorSwatch`.

A saturation panel and a hue slider, with an optional alpha slider and preset
swatches. The value is a `ColorCode`, which parses from hex, `rgb()` or `hsl()`
and converts back to any of them.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{ColorCode, ColorPicker, SliderChangeEvent, Swatches};

#[component]
fn Demo() -> Element {
    let mut color = use_signal(|| ColorCode::hex(0x228be6));

    rsx! {
        ColorPicker {
            value: color(),
            oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            with_alpha: true,
            swatches: Swatches::labelled([("#fa5252", "Red"), ("#228be6", "Blue"), ("#40c057", "Green")]),
        }
    }
}
```

Parsing and printing a `ColorCode`:

```rust
use libero::components::{ColorCode, ColorFormat};

# fn main() -> Result<(), libero::components::ParseColorError> {
let color: ColorCode = "#228be6".parse()?;

assert_eq!(color.to_hex(), "#228be6");
assert_eq!(color.to_hexa(), "#228be6ff");
assert_eq!(color.to_rgb(), "rgb(34, 139, 230)");
assert_eq!(color.to_rgba(), "rgba(34, 139, 230, 1)");
assert_eq!(color.to_hsl(), "hsl(208, 80%, 52%)");
assert_eq!(color.to_hsla(), "hsla(208, 80%, 52%, 1)");
assert_eq!(color.to_format(ColorFormat::Rgba), color.to_rgba());
color.to_rgba_channels(); // (34, 139, 230, 1.0)
color.to_hsla_channels(); // (208.0, 0.8, 0.52, 1.0)
# Ok(())
# }
```

## Building a value

`ColorCode::hex(0x228be6)` and its siblings build one, and a theme `HexColor`
converts into one. It prints as `#rrggbb`, or `#rrggbbaa` when translucent,
which a `style` accepts.

## Props

### `ColorPicker`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Width, panel height, thumbs, preview and swatches. Swatches keep their size when `full_width` stretches the picker. |
| `radius` | `Size` | `xxl` | Corner radius of the swatches and the preview. `xs` makes them square. |
| `value` | `ColorCode` | required | The color. Pair it with `oninput`. |
| `oninput` | `EventHandler<SliderChangeEvent<ColorCode>>` | - | `Start` and `End` bracket a drag on the panel or a slider. A key press or a swatch click sends `Change`, then `End`, so saving on `End` is enough. |
| `with_alpha` | `bool` | `false` | Shows the alpha slider and a preview swatch beside it. Without it, the color is always opaque. |
| `swatches` | `Swatches` | - | Preset colors under the panel, as `ColorCode`s or CSS strings. A string that is no color is skipped with a warning. The swatch equal to `value` shows as picked. |
| `swatches_per_row` | `usize` | - | Caps how many swatches share a row. Unset, they wrap to fill the width, seven per row at the picker's own width. |
| `with_picker` | `bool` | `true` | `false` leaves only the swatches, a palette. Without `swatches` it draws nothing and warns. |
| `onswatchclick` | `EventHandler<ColorCode>` | - | A swatch was clicked. `oninput` fires with the same color first. |
| `full_width` | `bool` | `false` | Takes the container's width instead of the size step's. |
| `name` | `String` | - | Posts the color in a hidden input of that name. |
| `format` | `ColorFormat` | `hex, or hexa with alpha` | How the hidden input writes the color. `hex`, `hexa`, `rgb`, `rgba`, `hsl` or `hsla`. |
| `focusable` | `bool` | `true` | `false` keeps the thumbs and swatches out of the tab order, for a picker in a dropdown whose input must keep focus. |
| `saturation_label` | `String` | `color.saturation` | Names the saturation panel's thumb. Unset, the localization's `color.saturation`. |
| `hue_label` | `String` | `color.hue` | Names the hue slider's thumb. Unset, the localization's `color.hue`. |
| `alpha_label` | `String` | `color.alpha` | Names the alpha slider's thumb. Unset, the localization's `color.alpha`. |

### `HueSlider`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `f64` | required | The hue in degrees, 0 to 360. Pair it with `oninput`. |
| `oninput` | `EventHandler<SliderChangeEvent>` | - | Every new hue. |
| `size` | `Size` | `md` | Track height and thumb. |
| `disabled` | `bool` | `false` | Dims the slider and stops it moving. |
| `focusable` | `bool` | `true` | `false` keeps the thumb out of the tab order. |
| `aria_label` | `String` | - | Names the thumb. |

### `AlphaSlider`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `f64` | required | The alpha, 0.0 to 1.0. Pair it with `oninput`. |
| `color` | `ColorCode` | required | The color the track fades in. Its own alpha is ignored. |
| `oninput` | `EventHandler<SliderChangeEvent>` | - | Every new alpha. |
| `size` | `Size` | `md` | Track height and thumb. |
| `disabled` | `bool` | `false` | Dims the slider and stops it moving. |
| `focusable` | `bool` | `true` | `false` keeps the thumb out of the tab order. |
| `aria_label` | `String` | - | Names the thumb. |

### `ColorSwatch`

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ColorCode` | required | The color. A translucent one shows a checkerboard through. |
| `size` | `Size` | `md` | Diameter. |
| `radius` | `Size` | `xxl` | Corner radius. Round by default. |
| `with_shadow` | `bool` | `true` | A faint inner ring, so a color close to the background keeps an edge. |
| `onclick` | `EventHandler<MouseEvent>` | - | Makes the swatch a `<button>`. |
| `children` | `Element` | - | Drawn on the color, such as a check mark, in black or white, whichever reads. |

Like every component, each of them also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. The attributes land on the
root.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work. `ColorSwatch` is one element and has no parts.

### `ColorPicker`

| Part | `data-slot` | Description |
|---|---|---|
| `ColorPickerPart::Saturation` | `saturation` | The saturation and brightness panel. |
| `ColorPickerPart::Body` | `body` | The row under the panel: the sliders and the preview. |
| `ColorPickerPart::Sliders` | `sliders` | The column of the hue and alpha sliders. |
| `ColorPickerPart::Hue` | `hue` | The hue slider. |
| `ColorPickerPart::Alpha` | `alpha` | The alpha slider, with `with_alpha`. |
| `ColorPickerPart::Track` | `track` | Both sliders' gradient tracks. |
| `ColorPickerPart::Thumb` | `thumb` | Every handle: the panel's and the sliders'. |
| `ColorPickerPart::Preview` | `preview` | The current color beside the sliders, with `with_alpha`. |
| `ColorPickerPart::Swatches` | `swatches` | The row of preset swatches. |
| `ColorPickerPart::Swatch` | `swatch` | One preset swatch. |

### `HueSlider` and `AlphaSlider`

| Part | `data-slot` | Description |
|---|---|---|
| `ColorSliderPart::Track` | `track` | The gradient track. |
| `ColorSliderPart::Thumb` | `thumb` | The handle, filled with the color it points at. |

## Accessibility

### Libero handles

- Each thumb is a slider with the usual keys.
- The saturation panel meets the 24px target size of WCAG 2.5.8 at every size.
- The swatch equal to the value is pressed and checked.

### You must

- Name swatches with `Swatches::labelled`. By default they are named by their
  hex, which a screen reader spells out.
- Give a `ColorSwatch` with `onclick` an `aria-label`; without one it is just
  "button", and it warns.
- Name the color in text beside a plain `ColorSwatch`, or give it `role: "img"`
  and an `aria-label`: on its own it says nothing.

### Example

A brand color picker with `swatches: Swatches::labelled(..)` naming each
swatch "Ocean", "Forest" and so on: a screen reader reads "Ocean", not a hex
code spelled out, and the swatch equal to the value reads as pressed.

### Limits

- The hue and alpha tracks meet the 24px target size of WCAG 2.5.8 from `md`
  up, not at `sm` or `xs`.

## Theme defaults

`theme.color_picker` is a `ColorPickerDefaults` with `size`, `swatches_per_row`,
`radius`, and per size step `width`, `saturation_height`, `thumb_size`,
`preview_size`, `spacing` and `swatch_size`. `theme.color_swatch` is a
`ColorSwatchDefaults` with `size`, `radius` and `sizes`.
