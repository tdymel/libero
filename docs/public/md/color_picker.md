# ColorPicker

Crate: `libero`
Import: `use libero::components::{ColorPicker, ColorCode};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/color>
Index: [index.md](index.md) lists every other page
Description: A saturation panel and a hue slider, with an optional alpha slider and preset swatches, over one `ColorCode`. Also documents `HueSlider`, `AlphaSlider` and `ColorSwatch`.

A saturation panel and a hue slider, with an optional alpha slider and preset
swatches. The value is a `ColorCode`, which parses from hex, `rgb()` or `hsl()`
and converts back to any of them.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{ColorCode, ColorPicker, SliderChangeEvent};

#[component]
fn Demo() -> Element {
    let mut color = use_signal(|| ColorCode::hex(0x228be6));

    rsx! {
        ColorPicker {
            value: color(),
            oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            with_alpha: true,
            swatches: ["#fa5252", "#228be6", "#40c057"],
        }
    }
}
```

## `ColorCode`

One type for every CSS form. It parses from `#rgb`, `#rgba`, `#rrggbb`,
`#rrggbbaa` (the `#` optional), `rgb()`, `rgba()`, `hsl()` and `hsla()`, comma
or space separated, with alpha as a number or a percentage. It converts back to
any of them.

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

Constructors: `ColorCode::hex(0x228be6)`, `rgb`, `rgba`, `hsl`, `hsla`, `hsva`,
and `From<HexColor>` for a theme color. `Display` writes `#rrggbb`, or
`#rrggbbaa` when translucent, which a `style` accepts.

A `ColorCode` keeps its hue on greys, so the hue thumb stays put when the panel
is dragged onto white or black.

## `oninput`

Carries a `SliderChangeEvent<ColorCode>`, as on a `Slider`. `Start` and `End`
bracket a drag on the panel or a slider, and `Change` carries every color in
between. A key press and a swatch click send `Change`, then `End`. Read
`event.value()` for the color, and save on `End` to save once per choice.

## Alpha

`with_alpha: true` shows the alpha slider and a preview swatch beside the
sliders. Without it the color is opaque. The picker ignores any alpha on
`value`, and every color it sends is fully opaque.

## Swatches

`swatches` takes `ColorCode`s or CSS strings, so a list can come from config.
A string that is no color is skipped with a warning. Swatches wrap, and
`swatches_per_row` caps how many share a row.

`onswatchclick` fires after `oninput` when one is clicked.

The swatch equal to `value` is pressed (`aria-pressed`) and carries a check
mark. A swatch is named by its hex, and `Swatches::labelled` names them
instead.

```rust,ignore
ColorPicker {
    value: color(),
    oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
    swatches: Swatches::labelled([("#fa5252", "Red"), ("#40c057", "Green")]),
}
```

`with_picker: false` leaves only the swatches, a palette. Without `swatches`
that draws nothing and warns.

Seven swatches fit a row at each `size`. `full_width` stretches the picker, not
the swatches, so more fit per row. `radius` sets the corners of the swatches
and the preview, round by default and square at `xs`.

## Forms

`name` posts the color in a hidden input. `format` picks its text, `hex` by
default, `hexa` with alpha, or `rgb`, `rgba`, `hsl` or `hsla`.

## Accessibility

- The panel's thumb moves saturation with Left/Right and brightness with
  Up/Down, one percent per press, ten with Shift.
- The hue slider moves one degree per arrow, the alpha slider one percent.
  Both take Home, End, PageUp and PageDown like a `Slider`.
- Swatches are toggle buttons: the one equal to the value is pressed. They
  are named by their hex unless `Swatches::labelled` names them.

Name the three thumbs with `saturation_label`, `hue_label` and `alpha_label`.
Unset, the names and the announced values come from the localization's `color`
group.

`focusable: false` takes all of it out of the tab order and stops a drag from
focusing a thumb. It is for a picker inside a dropdown whose text input must
keep focus, as in `ColorField`.

The hue and alpha tracks meet the 24px target size of WCAG 2.5.8 from `md` up,
and not at `sm` or `xs`. The saturation panel meets it at every size.

## `HueSlider`, `AlphaSlider`, `ColorSwatch`

The picker's parts, public on their own.

```rust,ignore
HueSlider {
    value: hue(),                      // degrees, 0-360
    oninput: move |event: SliderChangeEvent| hue.set(event.value()),
    aria_label: "Hue",
}

AlphaSlider {
    value: alpha(),                    // 0.0-1.0
    color: ColorCode::hex(0x228be6),   // the color the track fades in
    oninput: move |event: SliderChangeEvent| alpha.set(event.value()),
    aria_label: "Opacity",
}

ColorSwatch { color: ColorCode::hex(0x228be6) }
ColorSwatch { color: "rgba(250, 82, 82, 0.4)".parse().unwrap() }
ColorSwatch { color: ColorCode::hex(0x40c057), onclick: move |_| {}, "✓" }
```

Both sliders are a `Slider` with a gradient track as tall as the thumb, and no
filled bar or bubble. `ColorSwatch` shows a checkerboard through a translucent
color, becomes a `<button>` with `onclick`, and draws `children` in black or
white, whichever reads on the color.

## Props

### `ColorPicker`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Width, panel height, thumbs, preview and swatches. Swatches keep their size when `full_width` stretches the picker. |
| `radius` | `Size` | `xxl` | Corner radius of the swatches and the preview. `xs` makes them square. |
| `value` | `ColorCode` | - | The color. Pair it with `oninput`. |
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
| `value` | `f64` | - | The hue in degrees, 0 to 360. Pair it with `oninput`. |
| `oninput` | `EventHandler<SliderChangeEvent>` | - | Every new hue. |
| `size` | `Size` | `md` | Track height and thumb. |
| `disabled` | `bool` | `false` | Dims the slider and stops it moving. |
| `focusable` | `bool` | `true` | `false` keeps the thumb out of the tab order. |
| `aria_label` | `String` | - | Names the thumb. |

### `AlphaSlider`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `f64` | - | The alpha, 0.0 to 1.0. Pair it with `oninput`. |
| `color` | `ColorCode` | - | The color the track fades in. Its own alpha is ignored. |
| `oninput` | `EventHandler<SliderChangeEvent>` | - | Every new alpha. |
| `size` | `Size` | `md` | Track height and thumb. |
| `disabled` | `bool` | `false` | Dims the slider and stops it moving. |
| `focusable` | `bool` | `true` | `false` keeps the thumb out of the tab order. |
| `aria_label` | `String` | - | Names the thumb. |

### `ColorSwatch`

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ColorCode` | - | The color. A translucent one shows a checkerboard through. |
| `size` | `Size` | `md` | Diameter. |
| `radius` | `Size` | `xxl` | Corner radius. Round by default. |
| `with_shadow` | `bool` | `true` | A faint inner ring, so a color close to the background keeps an edge. |
| `onclick` | `EventHandler<MouseEvent>` | - | Makes the swatch a `<button>`. |
| `children` | `Element` | - | Drawn on the color, such as a check mark, in black or white, whichever reads. |

Like every component, each of them also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. The attributes land on the
root.

## Theme defaults

`theme.color_picker` is a `ColorPickerDefaults` with `size`, `swatches_per_row`,
`radius`, and per size step `width`, `saturation_height`, `thumb_size`,
`preview_size`, `spacing` and `swatch_size`. `theme.color_swatch` is a
`ColorSwatchDefaults` with `size`, `radius` and `sizes`.
