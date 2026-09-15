# ColorPicker

Crate: `libero`
Import: `use libero::components::{ColorPicker, ColorCode};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/color>
Index: [index.md](index.md) - every other component's markdown page
Description: A saturation panel and a hue slider, with an optional alpha slider and preset swatches, holding one `ColorCode` that converts to any CSS form. Also documents `HueSlider`, `AlphaSlider` and `ColorSwatch`.

Modelled on Mantine's `ColorPicker`, prop for prop, except where Rust does
better: the value is a typed `ColorCode`, not a string.

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
`#rrggbbaa` (the `#` optional), `rgb()`, `rgba()`, `hsl()` and `hsla()` - comma
or space separated, alpha as a number or a percentage - and converts back to
any of them:

```rust,ignore
let color: ColorCode = "#228be6".parse()?;

color.to_hex();    // "#228be6"
color.to_hexa();   // "#228be6ff"
color.to_rgb();    // "rgb(34, 139, 230)"
color.to_rgba();   // "rgba(34, 139, 230, 1)"
color.to_hsl();    // "hsl(208, 80%, 52%)"
color.to_hsla();   // "hsla(208, 80%, 52%, 1)"
color.to_format(ColorFormat::Rgba);
color.to_rgba_channels(); // (34, 139, 230, 1.0)
color.to_hsla_channels(); // (208.0, 0.8, 0.52, 1.0)
```

Constructors: `ColorCode::hex(0x228be6)`, `rgb`, `rgba`, `hsl`, `hsla`, `hsva`,
and `From<HexColor>` for a theme color. `Display` writes `#rrggbb`, or
`#rrggbbaa` when translucent - what a `style` accepts.

It is stored as hue, saturation, value and alpha. RGB has no hue on a grey, so
a picker holding RGB would lose its hue - and jump its hue thumb to red - the
moment the panel is dragged onto white or black. Stored as HSVA, the hue
survives as long as the caller keeps the value it was handed.

## `oninput`

Carries a `SliderChangeEvent<ColorCode>`, the `Slider` shape: `Start` and `End`
bracket a drag on the panel or on a slider, `Change` carries every color in
between. A key press and a swatch click emit `Change` then `End`, because
either settles on its color at once. Read `event.value()` for the color; match
on the phase to commit only on `End` - every way of choosing a color ends
there.

## Alpha

`with_alpha: true` shows the alpha slider and a preview swatch beside the
sliders. Without it the color is solid: any alpha on `value` is ignored when
rendering, and every color the picker emits - from the panel, the hue slider
or a swatch - is fully opaque.

## Swatches

`swatches` takes `ColorCode`s or CSS strings, parsed at runtime - a list from
config or a design token file. A string that is no color is skipped with a dev
warning, never a panic. Swatches wrap, so a
`full_width` picker fills its width with them. `swatches_per_row` caps how many
share a row.

`onswatchclick` fires after `oninput` when one is clicked.

The swatch equal to `value` is pressed (`aria-pressed`) and carries a check
mark. A swatch is named by its hex; `Swatches::labelled` gives each one a name
instead:

```rust,ignore
ColorPicker {
    value: color(),
    oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
    swatches: Swatches::labelled([("#fa5252", "Red"), ("#40c057", "Green")]),
}
```

`with_picker: false` leaves only the swatches - a palette. Without `swatches`
that draws nothing, and a debug build warns about it.

A swatch has a fixed size per `size` step, chosen so seven of them and their
gaps fit the step's width. `full_width` stretches the picker, not the swatches:
more of them fit per row instead.
`radius` sets the corners of the swatches and the preview: round by default,
`xs` for squares.

## Forms

`name` emits a hidden input, so the color posts with a form. `format` decides
its text: `hex` by default, `hexa` with alpha, or any of `rgb`, `rgba`, `hsl`,
`hsla`.

## Accessibility

- The panel's thumb moves saturation with Left/Right and brightness with
  Up/Down, one percent per press, ten with Shift.
- The hue slider moves one degree per arrow; the alpha slider one percent.
  Both take Home, End, PageUp and PageDown like a `Slider`.
- Swatches are toggle buttons: the one equal to the value is pressed. They
  are named by their hex unless `Swatches::labelled` names them.

Name the three thumbs with `saturation_label`, `hue_label` and `alpha_label`.
Unset, the names and the announced values come from the localization's `color`
group.

`focusable: false` takes all of it out of the tab order and stops a drag from
focusing a thumb. It exists for a picker inside a dropdown whose text input must
keep focus - which is how `ColorField` uses it.

A press anywhere on the hue or alpha track moves its thumb there, so the target
is the track, not just the thumb - and the track is only as tall as its thumb,
so its height is what decides. They meet WCAG 2.5.8 (target size) from `md`
upward and not below it: the two tracks sit 24px apart centre to centre at `md`,
18px at `sm` and 12px at `xs`, so at the two smallest steps neither the height
nor that spacing reaches the 24px the criterion wants. Choose `md` or larger
where 2.5.8 has to be met.

The saturation panel is its own target - a press anywhere in it moves the thumb,
and the panel is far larger than 24x24 at every size - so it meets 2.5.8
throughout. See [accessibility.md](accessibility.md) for the library's overall
position.

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

Both sliders render the same engine as `Slider`, with a gradient track as tall
as the thumb and no filled bar or value bubble. `ColorSwatch` shows a
checkerboard through a translucent color, becomes a `<button>` with `onclick`,
and draws `children` in black or white, whichever reads on the color.

## Not like Mantine

| Mantine | libero | Why |
|---|---|---|
| `value: string` | `value: ColorCode` | typed; convert afterwards |
| `format` decides whether alpha shows | `with_alpha` | the value has no format; `format` is only the posted text |
| `defaultValue` | controlled only | the `Slider` convention |
| `onChange` + `onChangeEnd` | one `oninput` with `Start`/`Change`/`End` | the `Slider` convention |
| `hiddenInputProps` | none | add on demand |

## Props

### ColorPicker

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `ColorCode` | required | Strictly controlled. |
| `oninput` | `EventHandler<SliderChangeEvent<ColorCode>>` | | Every new color, bracketed by `Start`/`End` on a drag. A key press or a swatch click emits `Change` then `End`. |
| `with_alpha` | `bool` | `false` | Alpha slider and preview swatch. |
| `swatches` | `Swatches` | | Preset colors; `ColorCode`s or CSS strings, or `Swatches::labelled` for named ones. The one equal to `value` is pressed. |
| `swatches_per_row` | `usize` | | Caps swatches per row; unset, they wrap. |
| `with_picker` | `bool` | `true` | `false` leaves only the swatches; warns in debug without any. |
| `onswatchclick` | `EventHandler<ColorCode>` | | A swatch was clicked, after `oninput`. |
| `full_width` | `bool` | `false` | The container's width instead of the size step's. |
| `size` | `Size` | `md` | Width, panel height, thumbs, preview and swatch size. |
| `radius` | `Size` | `xxl` | Corners of the swatches and the preview. |
| `name` | `String` | | Hidden input, so the color posts. |
| `format` | `ColorFormat` | `hex` / `hexa` | The hidden input's text. |
| `focusable` | `bool` | `true` | `false` for a picker in a dropdown. |
| `saturation_label` | `String` | `color.saturation` | Names the panel's thumb. |
| `hue_label` | `String` | `color.hue` | Names the hue thumb. |
| `alpha_label` | `String` | `color.alpha` | Names the alpha thumb. |

Plus `class`, `sx`, `states` and global attributes on the root.

### HueSlider / AlphaSlider

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `f64` | required | Degrees `0-360` / alpha `0.0-1.0`. |
| `color` | `ColorCode` | required | `AlphaSlider` only: the color the track fades in. |
| `oninput` | `EventHandler<SliderChangeEvent>` | | Every new value. |
| `size` | `Size` | `md` | Track height and thumb. |
| `disabled` | `bool` | `false` | Dims it and stops it moving. |
| `focusable` | `bool` | `true` | `false` keeps the thumb out of the tab order. |
| `aria_label` | `String` | | Names the thumb. |

### ColorSwatch

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ColorCode` | required | The color. |
| `size` | `Size` | `md` | Diameter. |
| `radius` | `Size` | `xxl` | Corner radius; round by default. |
| `with_shadow` | `bool` | `true` | Faint inner ring. |
| `onclick` | `EventHandler<MouseEvent>` | | Makes it a `<button>`. |
| `children` | `Element` | | Drawn on the color in black or white. |

## Theme

`Theme::color_picker` (`ColorPickerDefaults`): `size`, `swatches_per_row`,
`radius`, and per size step `width`, `saturation_height`, `thumb_size`,
`preview_size`, `spacing`, `swatch_size`. `Theme::color_swatch` (`ColorSwatchDefaults`): `size`, `radius`,
`sizes`.
