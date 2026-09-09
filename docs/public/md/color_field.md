# ColorField

Crate: `libero`
Import: `use libero::components::{ColorField, ColorCode};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/color/color_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A text field holding a `ColorCode`, with a preview swatch, an eyedropper and a `ColorPicker` in a dropdown.

Modelled on Mantine's `ColorInput`. The value and the dropdown are
[ColorPicker](color_picker.md)'s, so read that page for `ColorCode`, swatches
and the event phases.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{ColorCode, ColorField, SliderChangeEvent};

#[component]
fn Demo() -> Element {
    let mut color = use_signal(|| ColorCode::hex(0x228be6));

    rsx! {
        ColorField {
            label: "Brand color",
            value: color(),
            oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
            name: "brand",
        }
    }
}
```

## Typing

The input accepts every form `ColorCode` parses - hex, `rgb()`, `hsl()`, with
or without alpha - whatever `format` is. Text is kept exactly as typed while the
field has focus, and every time it parses the color is emitted as `Change`
then `End`, as a swatch or a key press in the dropdown is. On blur the text
goes back to `value` in `format`.

Text that does not parse is dropped on blur by default (`fix_on_blur: true`).
`fix_on_blur: false` keeps it on screen, so a caller can show a status for it;
`value` is still the last color that parsed.

`disallow_input: true` makes the input read-only: the dropdown is the only way
to change the color.

## `format` and posting

`format` is how the text shows the color: `hex` by default, `hexa` with
`with_alpha`, or any of `rgb`, `rgba`, `hsl`, `hsla`. The field extends the
input's attributes, so `name` lands on the input itself and it posts that text.

## Alpha

`with_alpha: true` adds the alpha slider to the dropdown. Without it the field
has no way to show or change alpha, so a translucent color typed or picked from
the screen arrives opaque.

## The dropdown

Focusing or clicking the input opens it; Escape and blur close it. It holds a
`ColorPicker` with the field's `size`, `with_alpha`, `swatches`,
`swatches_per_row` and `with_picker`. `with_picker: false` leaves only the
swatches, and no dropdown at all without any.
`close_on_swatch_click: true` closes it when a swatch is picked.

A mousedown in the dropdown is cancelled, so a drag never blurs the field and a
mouse user's focus stays in the text.

## Accessibility

The input is a `combobox` whose dropdown is a non-modal `dialog` named "Choose
color". Focus opens it
and stays in the text input, so typing works at once.

| Key | In the text input | In the dropdown |
|---|---|---|
| Arrow Down | moves focus into the picker: the saturation area, or the first swatch | the picker's own keys ([ColorPicker](color_picker.md#accessibility)) |
| Escape | closes the dropdown | closes it and returns focus to the text |

Picking a swatch that closes the dropdown returns focus to the text too. Focus
leaving both the text and the dropdown closes it.

## Preview and eyedropper

`with_preview` (on by default) draws the color as a swatch in the leading slot.

`with_eye_dropper` (on by default) adds a button in the trailing slot that picks
a color off the screen through the browser's `EyeDropper` API. It only renders
where that API exists - Chromium-based browsers today - and never under Blitz.
It is looked up after mount, so a server render and the hydrating client agree.
With `with_alpha` a picked color keeps the field's current alpha, without it the
color is opaque; a dismissed pick changes nothing.

## Not like Mantine

| Mantine | libero | Why |
|---|---|---|
| `value: string` | `value: ColorCode` | typed; convert afterwards |
| `format` decides whether alpha shows | `with_alpha` | the value has no format |
| `defaultValue` | controlled only | the `Slider` convention |
| `onChange` + `onChangeEnd` | one `oninput` with `Start`/`Change`/`End` | the `Slider` convention |
| `leftSection` / `rightSection` | `with_preview` / `with_eye_dropper` only | add on demand |
| `popoverProps`, `eyeDropperButtonProps` | none | add on demand |

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `ColorCode` | required | Strictly controlled. |
| `oninput` | `EventHandler<SliderChangeEvent<ColorCode>>` | | Every new color. A drag in the dropdown brackets its moves with `Start`/`End`; typed text that parses, a key press, a swatch and the eyedropper emit `Change` then `End`. |
| `format` | `ColorFormat` | `hex` / `hexa` | The text's form, and what posts. |
| `with_alpha` | `bool` | `false` | Alpha slider; keeps typed alpha. |
| `swatches` | `Swatches` | | Preset colors in the dropdown. |
| `swatches_per_row` | `usize` | | Caps swatches per row; unset, they wrap. |
| `with_picker` | `bool` | `true` | `false` leaves only the swatches. |
| `with_preview` | `bool` | `true` | Swatch in the leading slot. |
| `with_eye_dropper` | `bool` | `true` | Eyedropper button, where supported. |
| `disallow_input` | `bool` | `false` | Read-only text. |
| `fix_on_blur` | `bool` | `true` | Drops unparsable text on blur. |
| `close_on_swatch_click` | `bool` | `false` | A swatch closes the dropdown. |
| `placeholder` | `String` | | Shown while the text is empty. |
| `size` | `Size` | `md` | Control height, font and the picker. |
| `radius` | `Size` | `sm` | Frame corner radius. |
| `label` / `description` / `helper` | `Caption` | | The field slots. |
| `status` | `FieldStatus` | `Valid` | Validation state. |
| `required` | `bool` | `false` | `required` plus an asterisk. |
| `disabled` | `bool` | `false` | No typing, no dropdown. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. |

Plus `class`, `sx`, `states` and every `input` attribute.

## Theme

`Theme::color_field` (`ColorFieldDefaults`): `size`, `radius`, `with_preview`,
`with_eye_dropper`, `fix_on_blur`, `close_on_swatch_click`. The dropdown's
picker reads `Theme::color_picker`.
