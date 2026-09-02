use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{
    AlphaSlider, Code, CodeBlock, ColorCode, ColorPicker, ColorSwatch, Flex, HueSlider,
    SliderChangeEvent, Swatches, Text,
};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

/// Mantine's own default swatches.
const SWATCHES: [&str; 14] = [
    "#2e2e2e", "#868e96", "#fa5252", "#e64980", "#be4bdb", "#7950f2", "#4c6ef5", "#228be6",
    "#15aabf", "#12b886", "#40c057", "#82c91e", "#fab005", "#fd7e14",
];

const HUE_SLIDER: &str = r#"let mut hue = use_signal(|| 200.0);

HueSlider {
    value: hue(),
    oninput: move |event: SliderChangeEvent| hue.set(event.value()),
    aria_label: "Hue",
}"#;

const ALPHA_SLIDER: &str = r#"let mut alpha = use_signal(|| 0.6);

AlphaSlider {
    value: alpha(),
    color: ColorCode::hex(0x228be6),
    oninput: move |event: SliderChangeEvent| alpha.set(event.value()),
    aria_label: "Opacity",
}"#;

const COLOR_SWATCH: &str = r#"ColorSwatch { color: ColorCode::hex(0x228be6) }
ColorSwatch { color: "rgba(250, 82, 82, 0.4)".parse().unwrap() }
ColorSwatch { color: ColorCode::hex(0x40c057), onclick: move |_| {}, "✓" }"#;

const CONVERSIONS: &str = r##"let color: ColorCode = "#228be6".parse()?;

color.to_hex();   // "#228be6"
color.to_rgba();  // "rgba(34, 139, 230, 1)"
color.to_hsl();   // "hsl(208, 80%, 52%)"
color.to_format(ColorFormat::Hsla);
color.to_rgba_channels(); // (34, 139, 230, 1.0)"##;

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

#[component]
pub fn ColorPickerPage() -> Element {
    rsx! {
        DocPage {
            title: "ColorPicker",
            source: "libero/src/components/form/color",
            markdown: "/md/color_picker.md",
            properties: vec![
                props("ColorPicker", vec![
                    prop("value", "ColorCode")
                        .doc("Strictly controlled - pair it with `oninput`."),
                    prop("oninput", "EventHandler<SliderChangeEvent<ColorCode>>")
                        .doc("`Start`/`End` bracket a drag on the panel or a slider. A key press or a swatch click emits a lone `Change`."),
                    prop("with_alpha", "bool")
                        .default("false")
                        .doc("Shows the alpha slider and the preview swatch beside it."),
                    prop("swatches", "Swatches")
                        .doc("Preset colors under the panel. Takes `ColorCode`s or CSS strings; a string that is no color is skipped with a warning."),
                    prop("swatches_per_row", "usize").doc("Caps how many swatches share a row. Unset, they wrap to fill the width - seven per row at a size step's own width."),
                    prop("with_picker", "bool")
                        .default("true")
                        .doc("`false` leaves only the swatches - a palette. Without `swatches` it draws nothing and warns in a debug build."),
                    prop("onswatchclick", "EventHandler<ColorCode>")
                        .doc("A swatch was clicked. `oninput` fires with the same color first."),
                    prop("full_width", "bool")
                        .default("false")
                        .doc("Takes the container's width instead of the size step's."),
                    prop("size", "Size").default("md").doc("Width, panel height, thumbs, preview and swatches. Swatches keep their size when `full_width` stretches the picker."),
                    prop("radius", "Size").default("xxl").doc("Corner radius of the swatches and the preview - `xs` for squares."),
                    prop("name", "String")
                        .doc("Emits a hidden input of that name, so the color posts with a form."),
                    prop("format", "ColorFormat")
                        .default("hex, or hexa with alpha")
                        .doc("How the hidden input writes the color: `hex`, `hexa`, `rgb`, `rgba`, `hsl` or `hsla`."),
                    prop("focusable", "bool")
                        .default("true")
                        .doc("`false` keeps the thumbs and swatches out of the tab order, for a picker in a dropdown whose input must keep focus."),
                    prop("saturation_label", "String").doc("Names the saturation panel's thumb."),
                    prop("hue_label", "String").doc("Names the hue slider's thumb."),
                    prop("alpha_label", "String").doc("Names the alpha slider's thumb."),
                ]),
                props("HueSlider", vec![
                    prop("value", "f64").doc("Degrees, `0-360`. Strictly controlled."),
                    prop("oninput", "EventHandler<SliderChangeEvent>").doc("Every new hue."),
                    prop("size", "Size").default("md").doc("Track height and thumb."),
                    prop("disabled", "bool").default("false").doc("Dims the slider and stops it moving."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the thumb out of the tab order."),
                    prop("aria_label", "String").doc("Names the thumb."),
                ]),
                props("AlphaSlider", vec![
                    prop("value", "f64").doc("`0.0-1.0`. Strictly controlled."),
                    prop("color", "ColorCode").doc("The color the track fades in. Its own alpha is ignored."),
                    prop("oninput", "EventHandler<SliderChangeEvent>").doc("Every new alpha."),
                    prop("size", "Size").default("md").doc("Track height and thumb."),
                    prop("disabled", "bool").default("false").doc("Dims the slider and stops it moving."),
                    prop("focusable", "bool").default("true").doc("`false` keeps the thumb out of the tab order."),
                    prop("aria_label", "String").doc("Names the thumb."),
                ]),
                props("ColorSwatch", vec![
                    prop("color", "ColorCode").doc("The color. A translucent one shows a checkerboard through."),
                    prop("size", "Size").default("md").doc("Diameter."),
                    prop("radius", "Size").default("xxl").doc("Corner radius - round by default."),
                    prop("with_shadow", "bool").default("true").doc("A faint inner ring, so a color close to the background keeps an edge."),
                    prop("onclick", "EventHandler<MouseEvent>").doc("Makes the swatch a `<button>`."),
                    prop("children", "Element").doc("Drawn on the color, in black or white - whichever reads."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A saturation panel and a hue slider, with an optional alpha slider and preset swatches. "
                    "Controlled: it renders "
                    Code { source: "value" }
                    " and asks for the next color through "
                    Code { source: "oninput" }
                    "."
                }
                Text {
                    "The value is a "
                    Code { source: "ColorCode" }
                    ", one type for every CSS form. It parses from hex, "
                    Code { source: "rgb()" }
                    " or "
                    Code { source: "hsl()" }
                    " text and converts back to any of them, so the picker has no format to choose."
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
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::slider("radius", SIZES).default("xxl"),
                    Control::switch("with_alpha"),
                    Control::switch("swatches").code(|_, values| match is_on(values, "swatches") {
                        // The whole list, seven to a line, so the snippet
                        // needs no constant of ours.
                        true => {
                            let rows = SWATCHES
                                .chunks(7)
                                .map(|row| {
                                    let row: Vec<String> = row.iter().map(|c| format!("{c:?}")).collect();
                                    format!("    {}", row.join(", "))
                                })
                                .collect::<Vec<_>>();
                            vec![format!("swatches: [\n{},\n]", rows.join(",\n"))]
                        }
                        false => vec![],
                    }),
                    Control::switch("with_picker").default("true").code(|_, values| {
                        match is_on(values, "with_picker") {
                            true => vec![],
                            false => vec!["with_picker: false".to_string()],
                        }
                    }),
                    Control::switch("full_width"),
                ],
                render: move |values: DemoValues| rsx! {
                    ColorPickerDemo { values }
                },
            }
            DocSection {
                title: "Converting the value",
                Text {
                    "A "
                    Code { source: "ColorCode" }
                    " is stored as hue, saturation, value and alpha. That is what keeps the hue thumb in place "
                    "when the panel is dragged onto grey or black, where RGB has no hue at all."
                }
                CodeBlock { source: CONVERSIONS, language: "rust" }
            }
            DocSection {
                title: "HueSlider",
                Text {
                    "The picker's hue track, on its own. Its value is plain degrees."
                }
                HueSliderPreview {}
                CodeBlock { source: HUE_SLIDER, language: "rust" }
            }
            DocSection {
                title: "AlphaSlider",
                Text {
                    "The picker's opacity track: transparent to "
                    Code { source: "color" }
                    " over a checkerboard."
                }
                AlphaSliderPreview {}
                CodeBlock { source: ALPHA_SLIDER, language: "rust" }
            }
            DocSection {
                title: "ColorSwatch",
                Text {
                    "A patch of one color, which is what the picker's preview and swatches are drawn with. "
                    "With "
                    Code { source: "onclick" }
                    " it is a button."
                }
                Flex { direction: "row", gap: "sm", align: "center",
                    ColorSwatch { color: ColorCode::hex(0x228be6) }
                    ColorSwatch { color: ColorCode::rgba(250, 82, 82, 0.4) }
                    ColorSwatch { color: ColorCode::hex(0x40c057), onclick: move |_| {}, "✓" }
                }
                CodeBlock { source: COLOR_SWATCH, language: "rust" }
            }
        }
    }
}

/// Its own component so the color survives a control change.
#[component]
fn ColorPickerDemo(values: DemoValues) -> Element {
    let mut color = use_signal(|| ColorCode::hex(0x228be6));

    let swatches = match is_on(&values, "swatches") {
        true => Swatches::from(SWATCHES),
        false => Swatches::default(),
    };
    let full_width = is_on(&values, "full_width");

    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%"),
            ColorPicker {
                value: color(),
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
                size: values.str("size"),
                radius: values.str("radius"),
                with_picker: is_on(&values, "with_picker"),
                with_alpha: is_on(&values, "with_alpha"),
                full_width,
                swatches,
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
    rsx! {
        Flex { direction: "column", gap: "xs", sx: libero::sx::sx().width("240px"),
            AlphaSlider {
                value: alpha(),
                color: ColorCode::hex(0x228be6),
                oninput: move |event: SliderChangeEvent| alpha.set(event.value()),
                aria_label: "Opacity",
            }
            Text { size: "sm", "{alpha()}" }
        }
    }
}
