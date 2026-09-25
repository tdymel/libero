# ColorField

Crate: `libero`
Import: `use libero::components::{ColorField, ColorCode};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/color/color_field.rs>
Index: [index.md](index.md) lists every other page
Description: A text field holding a `ColorCode`, with a preview swatch, an eyedropper and a `ColorPicker` in a dropdown.

A text field holding a `ColorCode`, with a preview swatch, an eyedropper and a
[ColorPicker](color_picker.md) in a dropdown. The text accepts every form a
`ColorCode` parses, such as hex, `rgb()` or `hsl()`. It stays as typed while
the field has focus, each parse sends the color, and on blur it shows `value`
in `format`.

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
            swatches: ["#2e2e2e", "#fa5252", "#228be6", "#12b886"],
            value: color(),
            oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
        }
    }
}
```

## Props

### `ColorField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `ColorCode` | - | Strictly controlled. Pair it with `oninput`. |
| `oninput` | `EventHandler<SliderChangeEvent<ColorCode>>` | - | Every new color. A drag in the dropdown sends `Start` and `End` around its moves. A key press, a swatch, typed text that parses and the eyedropper send `Change` then `End`. |
| `validate` | `Validators<ColorCode>` | - | Rules over the color, shown once the field loses focus or its form is submitted. |
| `format` | `ColorFormat` | hex, or hexa with alpha | How the text shows the color, and so what the field posts. Typing accepts every form either way. |
| `with_alpha` | `bool` | `false` | Shows the alpha slider in the dropdown. Without it a translucent color arrives opaque. |
| `swatches` | `Swatches` | - | Preset colors in the dropdown. Takes `ColorCode`s or CSS strings. |
| `swatches_per_row` | `usize` | - | Caps how many swatches share a row. Unset, they wrap to fill the width. |
| `with_picker` | `bool` | `true` | `false` leaves only the swatches in the dropdown, and no dropdown without them. |
| `with_preview` | `bool` | `true` | Shows the color as a swatch in the leading slot. |
| `with_eye_dropper` | `bool` | `true` | Adds a button in the trailing slot that picks a color off the screen. Shown only where the browser supports it, Chromium today. |
| `disallow_input` | `bool` | `false` | Makes the text read-only, so a color comes from the dropdown alone. |
| `fix_on_blur` | `bool` | `true` | Text that does not parse goes back to the last valid color on blur. Off, it stays and shows `color.invalid` as an error. |
| `close_on_swatch_click` | `bool` | `false` | Picking a swatch closes the dropdown. |
| `name` | `FieldName<ColorCode>` | - | What the field posts as, the text in `format`. A path such as `Theme::FIELDS.accent()` also binds the color to the surrounding `Form`'s value when the field has no `oninput`. |
| `placeholder` | `String` | - | Shown while the text is empty. |
| `size` | `Size` | `md` | Control height, font size and the dropdown's picker. |
| `radius` | `Size` | `sm` | Corner radius of the frame. |
| `label` | `Caption` | - | The field's caption. |
| `description` | `Caption` | - | Between the label and the control. |
| `helper` | `Caption` | - | Under the control. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables typing and the dropdown, and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. |
| `dropdown_parts` | `Parts<ColorDropdownPart>` | - | Styles the portaled dropdown and the picker in it. |

`ColorField` also takes the `<input>` HTML attributes and, like every
component, the shared props `sx`, `class`, `style`, `states`, and any extra
HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FieldPart::Label` | `label` | The label above the control. |
| `FieldPart::Required` | `required` | The required asterisk, in the label. |
| `FieldPart::Description` | `description` | The caption between the label and the control. |
| `FieldPart::Frame` | `frame` | The bordered box around the control. |
| `FieldPart::Leading` | `leading` | The slot before the control: an icon, a prefix. |
| `FieldPart::Control` | `control` | The element the label names. |
| `FieldPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

### Dropdown

The dropdown is portaled out of the field, so its parts take the
`dropdown_parts` prop. They match from the dropdown box at any depth.

| Part | `data-slot` | Description |
|---|---|---|
| `ColorDropdownPart::Panel` | `dropdown` | The dropdown box itself. |
| `ColorDropdownPart::Saturation` | `saturation` | The saturation and brightness panel. |
| `ColorDropdownPart::Body` | `body` | The row under the panel: the sliders and the preview. |
| `ColorDropdownPart::Sliders` | `sliders` | The column of the hue and alpha sliders. |
| `ColorDropdownPart::Hue` | `hue` | The hue slider. |
| `ColorDropdownPart::Alpha` | `alpha` | The alpha slider, with `with_alpha`. |
| `ColorDropdownPart::Track` | `track` | Both sliders' gradient tracks. |
| `ColorDropdownPart::Thumb` | `thumb` | Every handle: the panel's and the sliders'. |
| `ColorDropdownPart::Preview` | `preview` | The current color beside the sliders, with `with_alpha`. |
| `ColorDropdownPart::Swatches` | `swatches` | The row of preset swatches. |
| `ColorDropdownPart::Swatch` | `swatch` | One preset swatch. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down` | Moves focus into the picker, onto the saturation area or the first swatch, where the [ColorPicker](color_picker.md#accessibility) keys apply. |
| `Escape` | Moves focus back to the text. |
| `Tab` or `Shift+Tab` | Past either end of the dropdown: leaves the field. |

### Libero handles

- Focus opens the dropdown and stays in the text, so typing works at once.
- A swatch that closes the dropdown moves focus back to the text.
- Focus leaving both the text and the dropdown closes it.
- A mouse click in the dropdown leaves focus in the text.

## Theme defaults

`ColorFieldDefaults` holds `size`, `radius`, `with_preview`,
`with_eye_dropper`, `fix_on_blur` and `close_on_swatch_click`. The dropdown's
picker reads `ColorPickerDefaults`.
