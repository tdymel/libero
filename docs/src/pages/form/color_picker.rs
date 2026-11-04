use super::dropdown_parts::color_picker_parts;
use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::ColorSliderPart;
use libero::components::{
    AlphaSlider, Code, ColorCode, ColorPicker, ColorSwatch, Flex, HueSlider, SliderChangeEvent,
    Swatches, Text,
};
use libero::theme::{Size, responsive};
use libero::{use_formats, use_theme};

/// The demo swatches and their names.
const SWATCHES: [(&str, &str); 14] = [
    ("#2e2e2e", "Dark"),
    ("#868e96", "Gray"),
    ("#fa5252", "Red"),
    ("#e64980", "Pink"),
    ("#be4bdb", "Grape"),
    ("#7950f2", "Violet"),
    ("#4c6ef5", "Indigo"),
    ("#228be6", "Blue"),
    ("#15aabf", "Cyan"),
    ("#12b886", "Teal"),
    ("#40c057", "Green"),
    ("#82c91e", "Lime"),
    ("#fab005", "Yellow"),
    ("#fd7e14", "Orange"),
];

const HUE_SLIDER: &str = r#"let mut hue = use_signal(|| 200.0);

HueSlider {
    value: hue(),
    oninput: move |event: SliderChangeEvent| hue.set(event.value()),
    aria_label: "Hue",
}
Text { size: "sm", "{hue()}°" }"#;

const ALPHA_SLIDER: &str = r#"let mut alpha = use_signal(|| 0.6);
let formats = use_formats();

Text { size: "sm", "Opacity" }
AlphaSlider {
    value: alpha(),
    color: ColorCode::hex(0x228be6),
    oninput: move |event: SliderChangeEvent| alpha.set(event.value()),
    aria_label: "Opacity",
}
Text { size: "sm", {formats.write_percent(&(alpha() * 100.0).round().to_string())} }"#;

const COLOR_SWATCH: &str = r#"let mut picked = use_signal(|| false);

ColorSwatch { color: ColorCode::hex(0x228be6), role: "img", aria_label: "Blue" }
ColorSwatch { color: "rgba(250, 82, 82, 0.4)".parse().unwrap(), role: "img", aria_label: "Translucent red" }
ColorSwatch {
    color: ColorCode::hex(0xffffff),
    background: "conic-gradient(red, yellow, lime, aqua, blue, magenta, red)",
    role: "img",
    aria_label: "Hue wheel",
}
ColorSwatch {
    color: ColorCode::hex(0x40c057),
    onclick: move |_| picked.toggle(),
    aria_label: "Green",
    aria_pressed: picked().to_string(),
    if picked() { "✓" }
}"#;

/// The preview prints the value in three formats under the picker, which is
/// how the page shows that one `ColorCode` converts to every CSS form.
fn wrap_picker(values: &DemoValues, source: &str) -> String {
    match values.str("component").as_str() {
        "hue" => HUE_SLIDER.to_string(),
        "alpha" => ALPHA_SLIDER.to_string(),
        "swatch" => COLOR_SWATCH.to_string(),
        _ => format!(
            "let mut color = use_signal(|| ColorCode::hex(0x228be6));\n\n\
             Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\"),\n\
             {}    Text {{ size: \"sm\", \"{{color().to_hexa()}} · {{color().to_rgba()}} · {{color().to_hsla()}}\" }}\n}}",
            indent(source)
        ),
    }
}

fn picker(values: &DemoValues) -> bool {
    values.str("component") == "picker"
}

/// The switch only shows beside swatches: off alone would draw nothing.
fn with_picker(values: &DemoValues) -> bool {
    is_on(values, "with_picker") || !is_on(values, "swatches")
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

#[component]
pub fn ColorPickerPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "ColorPicker",
            source: "libero/src/components/form/color",
            markdown: "/md/color_picker.md",
            properties: vec![
                props("ColorPicker", vec![
                    prop("size", "Responsive<Size>").default(theme.color_picker.size.as_str()).doc("Width, panel height, thumbs, preview and swatches. Swatches keep their size when `full_width` stretches the picker. Or one per breakpoint, `size: responsive(Size::Sm).md(Size::Lg)`."),
                    prop("radius", "ThemeAwareValue").default(theme.color_picker.radius.as_str()).doc("Corner radius of the swatches and the preview, or any CSS, e.g. `radius: \"0\"`. `xs` makes them square."),
                    prop("value", "ColorCode").default("required")
                        .doc("The color. Pair it with `oninput`."),
                    prop("oninput", "EventHandler<SliderChangeEvent<ColorCode>>")
                        .doc("`Start` and `End` bracket a drag on the panel or a slider. A key press or a swatch click sends `Change`, then `End`, so saving on `End` is enough."),
                    prop("with_alpha", "bool")
                        .default("false")
                        .doc("Shows the alpha slider and a preview swatch beside it. Without it, the color is always opaque."),
                    prop("swatches", "Swatches")
                        .doc("Preset colors under the panel, as `ColorCode`s or CSS strings. A string that is no color is skipped with a warning. The swatch equal to `value` shows as picked."),
                    prop("swatches_per_row", "usize").doc("Caps how many swatches share a row. Unset, they wrap to fill the width, seven per row at the picker's own width."),
                    prop("with_picker", "bool")
                        .default("true")
                        .doc("`false` leaves only the swatches, a palette. Without `swatches` it draws nothing and warns. Its swatches are buttons: without `oninput` or `onswatchclick` they do nothing, and it warns once."),
                    prop("onswatchclick", "EventHandler<ColorCode>")
                        .doc("A swatch was clicked. `oninput` fires with the same color first."),
                    prop("full_width", "bool")
                        .default("false")
                        .doc("Takes the container's width instead of the size step's."),
                    prop("name", "String")
                        .doc("Posts the color in a hidden input of that name."),
                    prop("format", "ColorFormat")
                        .default("hex, or hexa with alpha")
                        .doc("How the hidden input writes the color. `hex`, `hexa`, `rgb`, `rgba`, `hsl` or `hsla`."),
                    prop("focusable", "bool")
                        .default("true")
                        .doc("`false` keeps the thumbs and swatches out of the tab order, for a picker in a dropdown whose input must keep focus."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Dims the panel, the sliders and the swatches and takes them out of the tab order. The hidden input no longer posts. A disabled `Fieldset` does the same."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Shows the color but ignores drags, keys and swatch clicks, `oninput` included. The thumbs stay tab stops and read as read-only; the swatches stay focusable buttons marked `aria-disabled`."),
                    prop("saturation_label", "String").default("color.saturation").doc("Names the saturation panel's thumb. Unset, the localization's `color.saturation`."),
                    prop("hue_label", "String").default("color.hue").doc("Names the hue slider's thumb. Unset, the localization's `color.hue`."),
                    prop("alpha_label", "String").default("color.alpha").doc("Names the alpha slider's thumb. Unset, the localization's `color.alpha`."),
                ])
                .parts("ColorPickerPart", color_picker_parts()),
                props("HueSlider", vec![
                    prop("value", "f64").default("required").doc("The hue in degrees, 0 to 360. Pair it with `oninput`. A `ColorPicker`'s own hue slider stops at 359: 360 is the same hue as 0."),
                    prop("oninput", "EventHandler<SliderChangeEvent>").doc("Every new hue."),
                    prop("size", "Responsive<Size>").default(theme.color_picker.size.as_str()).doc("Track height and thumb. Or one per breakpoint."),
                    prop("disabled", "bool").default("false").doc("Dims the slider and stops it moving."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the thumb out of the tab order."),
                    prop("aria_label", "String").doc("Names the thumb. Unset, the thumb has no name: unlike `ColorPicker`, the standalone slider falls back to no localization."),
                ])
                .parts("ColorSliderPart", vec![
                    (ColorSliderPart::Track, "The gradient track."),
                    (ColorSliderPart::Thumb, "The handle, filled with the color it points at."),
                ]),
                props("AlphaSlider", vec![
                    prop("value", "f64").default("required").doc("The alpha, 0.0 to 1.0. Pair it with `oninput`."),
                    prop("color", "ColorCode").default("required").doc("The color the track fades in. Its own alpha is ignored."),
                    prop("oninput", "EventHandler<SliderChangeEvent>").doc("Every new alpha."),
                    prop("size", "Responsive<Size>").default(theme.color_picker.size.as_str()).doc("Track height and thumb. Or one per breakpoint."),
                    prop("disabled", "bool").default("false").doc("Dims the slider and stops it moving."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the thumb out of the tab order."),
                    prop("aria_label", "String").doc("Names the thumb. Unset, the thumb has no name: unlike `ColorPicker`, the standalone slider falls back to no localization."),
                ])
                .parts("ColorSliderPart", vec![
                    (ColorSliderPart::Track, "The gradient track over a checkerboard."),
                    (ColorSliderPart::Thumb, "The handle, filled with the color at its alpha."),
                ]),
                props("ColorSwatch", vec![
                    prop("color", "ColorCode").default("required").doc("The color. A translucent one shows a checkerboard through."),
                    prop("size", "Size").default(theme.color_swatch.size.as_str()).doc("Diameter."),
                    prop("radius", "ThemeAwareValue").default(theme.color_swatch.radius.as_str()).doc("Corner radius, or any CSS, e.g. `radius: \"0\"`. Round by default."),
                    prop("background", "String").doc("Any CSS `background`, such as a conic gradient for a hue wheel, painted in place of `color`. `color` still decides whether the mark is black or white, so set it to what shows."),
                    prop("with_shadow", "bool").default("true").doc("A faint inner ring, so a color close to the background keeps an edge."),
                    prop("onclick", "EventHandler<MouseEvent>").doc("Makes the swatch a `<button>`."),
                    prop("children", "Element").doc("Drawn on the color, such as a check mark, in black or white, whichever reads."),
                ]),
            ],
            accessibility: a11y()
                .key(["ArrowRight", "ArrowDown"], "In the swatch row: the next swatch, wrapping to the first. The row is one list even where it wraps onto several lines, so Down is the next swatch, not the one below it.")
                .key(["ArrowLeft", "ArrowUp"], "In the swatch row: the previous swatch, wrapping to the last. Up is the previous swatch, not the one above it.")
                .key(["Home", "End"], "In the swatch row: the first or the last swatch.")
                .handles([
                    "Each thumb is a slider with the usual keys. In a `ColorPicker` the hue runs from 0 to 359 degrees, since 360 is the same hue as 0.",
                    "The saturation panel and the hue and alpha thumbs meet the 24px target size of WCAG 2.5.8 at every size. At `xs` and `sm` the two sliders sit further apart than `spacing`, so their 24px squares do not overlap.",
                    "The swatch equal to the value is pressed and checked.",
                    "The preset row is a group named \"Color swatches\" (localized, `ColorLabels::swatches`), unless you name the picker.",
                ])
                .must([
                    "Give a picker you name `role: \"group\"` beside its `aria-label`: the root is a plain `div` that takes no name, and the swatch row then drops its own.",
                    "Name swatches with `Swatches::labelled`. By default they are named by their hex, which a screen reader spells out.",
                    "Give a standalone `HueSlider` or `AlphaSlider` an `aria_label`. Otherwise screen readers announce an unnamed slider.",
                    "Give a `ColorSwatch` with `onclick` an `aria-label`; without one it is just \"button\", and it warns.",
                    "Name the color in text beside a plain `ColorSwatch`, or give it `role: \"img\"` and an `aria-label`: on its own it says nothing.",
                ])
                .example("A brand color picker with `swatches: Swatches::labelled(..)` naming each swatch \"Ocean\", \"Forest\" and so on: a screen reader reads \"Ocean\", not a hex code spelled out, and the swatch equal to the value reads as pressed."),
            lead: rsx! {
                Text {
                    "A saturation panel and a hue slider, with an optional alpha slider and "
                    "preset swatches. The value is a "
                    Code { source: "ColorCode" }
                    ", which parses from hex, "
                    Code { source: "rgb()" }
                    " or "
                    Code { source: "hsl()" }
                    " and converts back to any of them. It reads commas, spaces and "
                    Code { source: "/" }
                    " alike and clamps a value out of range, as CSS does, but knows no named colors: "
                    Code { source: "\"red\"" }
                    " is an error. "
                    Code { source: "ColorCode::hex(0x228be6)" }
                    " and its siblings build one, and a theme "
                    Code { source: "HexColor" }
                    " converts into one. It prints as "
                    Code { source: "#rrggbb" }
                    ", or "
                    Code { source: "#rrggbbaa" }
                    " when translucent, which a "
                    Code { source: "style" }
                    " accepts."
                }
                Text {
                    "To open a picker from a text field or from a swatch button, use "
                    Code { source: "ColorField" }
                    ": it brings the dropdown, the focus return and Escape, and "
                    Code { source: "variant: ColorFieldVariant::Swatch" }
                    " leaves only the swatch."
                }
            },
            Demo {
                component: "ColorPicker",
                children_text: "",
                // `value` is required, so the snippet needs it even though no
                // control sets it.
                fixed: vec![
                    "value: color()".to_string(),
                    "oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value())"
                        .to_string(),
                ],
                wrap: Wrap(wrap_picker),
                controls: vec![
                    // Not a prop: the picker or one of its standalone parts, printed whole.
                    Control::toggle("component", ["picker", "hue", "alpha", "swatch"])
                        .labels(["ColorPicker", "HueSlider", "AlphaSlider", "ColorSwatch"])
                        .code(|_, _| vec![]),
                    Control::sizes("size").default(theme.color_picker.size.as_str()).hidden_when(|values| !picker(values))
                        .code(|control, values| match (responsive_on(values), values.str("size")) {
                            (true, size) => vec![format!("size: responsive(Size::Sm).md(Size::{:?})", Size::from(size.as_str()))],
                            (false, size) if size == control.default => vec![],
                            (false, size) => vec![format!("size: {size:?}")],
                        }),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").hidden_when(|values| !picker(values)).code(|_, _| vec![]),
                    Control::sizes("radius").default(theme.color_picker.radius.as_str()).hidden_when(|values| !picker(values)),
                    Control::switch("with_alpha").hidden_when(|values| !picker(values) || !with_picker(values)),
                    Control::switch("swatches").hidden_when(|values| !picker(values)).code(|_, values| match is_on(values, "swatches") {
                        // The whole list, one swatch to a line, so the
                        // snippet needs no constant of ours.
                        true => {
                            let rows = SWATCHES
                                .iter()
                                .map(|(color, name)| format!("    ({color:?}, {name:?}),"))
                                .collect::<Vec<_>>();
                            vec![format!("swatches: Swatches::labelled([\n{}\n])", rows.join("\n"))]
                        }
                        false => vec![],
                    }),
                    // Without swatches, `with_picker: false` would draw nothing.
                    Control::switch("with_picker").default("true").hidden_when(|values| !picker(values) || !is_on(values, "swatches")).code(|_, values| {
                        match is_on(values, "with_picker") {
                            true => vec![],
                            false => vec!["with_picker: false".to_string()],
                        }
                    }),
                    Control::switch("full_width").hidden_when(|values| !picker(values)),
                    Control::switch("disabled").hidden_when(|values| !picker(values)),
                    Control::switch("readonly").hidden_when(|values| !picker(values)),
                ],
                render: move |values: DemoValues| match values.str("component").as_str() {
                    "hue" => rsx! { HueSliderPreview {} },
                    "alpha" => rsx! { AlphaSliderPreview {} },
                    "swatch" => rsx! { ColorSwatchPreview {} },
                    _ => rsx! { ColorPickerDemo { values } },
                },
            }
        }
    }
}

/// The demo's size from the `md` breakpoint up, `sm` below it.
fn responsive_on(values: &DemoValues) -> bool {
    values.str("responsive") == "true"
}

/// Its own component so the color survives a control change.
#[component]
fn ColorPickerDemo(values: DemoValues) -> Element {
    let mut color = use_signal(|| ColorCode::hex(0x228be6));

    let swatches = match is_on(&values, "swatches") {
        true => Swatches::labelled(SWATCHES),
        false => Swatches::default(),
    };
    let full_width = is_on(&values, "full_width");

    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%"),
            ColorPicker {
                value: color(),
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
                size: match (responsive_on(&values), Size::from(values.str("size").as_str())) {
                    (true, size) => responsive(Size::Sm).md(size),
                    (false, size) => size.into(),
                },
                radius: values.str("radius"),
                with_picker: with_picker(&values),
                with_alpha: is_on(&values, "with_alpha") && with_picker(&values),
                full_width,
                swatches,
                disabled: is_on(&values, "disabled"),
                readonly: is_on(&values, "readonly"),
                saturation_label: "Saturation",
                hue_label: "Hue",
                alpha_label: "Opacity",
            }
            Text { size: "sm", "{color().to_hexa()} · {color().to_rgba()} · {color().to_hsla()}" }
        }
    }
}

#[component]
fn HueSliderPreview() -> Element {
    let mut hue = use_signal(|| 200.0);
    rsx! {
        Flex { direction: "column", gap: "xs", sx: libero::sx::sx().width("240px"),
            HueSlider {
                value: hue(),
                oninput: move |event: SliderChangeEvent| hue.set(event.value()),
                aria_label: "Hue",
            }
            Text { size: "sm", "{hue()}°" }
        }
    }
}

#[component]
fn AlphaSliderPreview() -> Element {
    let mut alpha = use_signal(|| 0.6);
    let formats = use_formats();
    rsx! {
        Flex { direction: "column", gap: "xs", sx: libero::sx::sx().width("240px"),
            Text { size: "sm", "Opacity" }
            AlphaSlider {
                value: alpha(),
                color: ColorCode::hex(0x228be6),
                oninput: move |event: SliderChangeEvent| alpha.set(event.value()),
                aria_label: "Opacity",
            }
            Text { size: "sm", {formats.write_percent(&(alpha() * 100.0).round().to_string())} }
        }
    }
}

/// The clickable swatch toggles, so a press shows: its check and `aria-pressed` follow it.
#[component]
fn ColorSwatchPreview() -> Element {
    let mut picked = use_signal(|| false);
    rsx! {
        Flex { direction: "row", gap: "sm", align: "center",
            ColorSwatch { color: ColorCode::hex(0x228be6), role: "img", aria_label: "Blue" }
            ColorSwatch { color: ColorCode::rgba(250, 82, 82, 0.4), role: "img", aria_label: "Translucent red" }
            ColorSwatch {
                color: ColorCode::hex(0xffffff),
                background: "conic-gradient(red, yellow, lime, aqua, blue, magenta, red)",
                role: "img",
                aria_label: "Hue wheel",
            }
            ColorSwatch {
                color: ColorCode::hex(0x40c057),
                onclick: move |_| picked.toggle(),
                aria_label: "Green",
                aria_pressed: picked().to_string(),
                if picked() { "✓" }
            }
        }
    }
}
