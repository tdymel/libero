use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Code, ColorCode, ColorField, FieldStatus, Flex, SliderChangeEvent, Swatches, Text,
};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

const SWATCHES: [&str; 7] = [
    "#2e2e2e", "#fa5252", "#be4bdb", "#228be6", "#12b886", "#fab005", "#fd7e14",
];

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

#[component]
pub fn ColorFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "ColorField",
            source: "libero/src/components/form/color/color_field.rs",
            markdown: "/md/color_field.md",
            properties: vec![
                props("ColorField", vec![
                    prop("value", "ColorCode")
                        .doc("Strictly controlled - pair it with `oninput`."),
                    prop("oninput", "EventHandler<SliderChangeEvent<ColorCode>>")
                        .doc("A drag in the dropdown brackets its moves with `Start`/`End`, and a key press or a swatch in the dropdown emits `Change` then `End`. Typed text that parses and the eyedropper emit a lone `Change`."),
                    prop("format", "ColorFormat")
                        .default("hex, or hexa with alpha")
                        .doc("How the text shows the color, and so what `name` posts. Typing accepts every form either way."),
                    prop("with_alpha", "bool")
                        .default("false")
                        .doc("Shows the alpha slider in the dropdown. Without it a translucent color arrives opaque."),
                    prop("swatches", "Swatches").doc("Preset colors in the dropdown. Takes `ColorCode`s or CSS strings."),
                    prop("swatches_per_row", "usize").doc("Caps how many swatches share a row. Unset, they wrap to fill the width - seven per row at a size step's own width."),
                    prop("with_picker", "bool")
                        .default("true")
                        .doc("`false` leaves only the swatches in the dropdown, and no dropdown without them."),
                    prop("with_preview", "bool").default("true").doc("The swatch in the leading slot."),
                    prop("with_eye_dropper", "bool")
                        .default("true")
                        .doc("The eyedropper button in the trailing slot. Shown only where the platform has one - Chromium today."),
                    prop("disallow_input", "bool")
                        .default("false")
                        .doc("Makes the text read-only: a color comes from the dropdown alone."),
                    prop("fix_on_blur", "bool")
                        .default("true")
                        .doc("Text that does not parse goes back to the last valid color on blur."),
                    prop("close_on_swatch_click", "bool").default("false").doc("Picking a swatch closes the dropdown."),
                    prop("placeholder", "String").doc("Shown while the text is empty."),
                    prop("size", "Size").default("md").doc("Control height, font size and the dropdown picker."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
                    prop("label", "Caption").doc("The field's caption."),
                    prop("description", "Caption").doc("Between the label and the control."),
                    prop("helper", "Caption").doc("Under the control."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `required` to the input and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables typing and the dropdown, and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A text field holding a color, with a preview swatch, an eyedropper and a "
                    Code { source: "ColorPicker" }
                    " in a dropdown. Controlled through "
                    Code { source: "value" }
                    " and "
                    Code { source: "oninput" }
                    ", and the value is a "
                    Code { source: "ColorCode" }
                    " - the same one the picker holds."
                }
                Text {
                    "Typed text is kept as typed until the field blurs, and every time it parses the color is "
                    "emitted. The input keeps focus while the dropdown is used, so its blur is what closes it."
                }
            },
            Demo {
                component: "ColorField",
                children_text: "",
                controls: vec![
                    Control::slider("size", SIZES).default("md"),
                    Control::slider("radius", SIZES).default("sm"),
                    Control::select("format", ["hex", "hexa", "rgb", "rgba", "hsl", "hsla"])
                        .default("hex")
                        .code(|_, values| match values.str("format").as_str() {
                            "hex" => vec![],
                            format => vec![format!("format: {format:?}")],
                        }),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("Low contrast on white.".into())"#
                                    .to_string(),
                            ],
                            "error" => vec![r#"status: "Pick a brand color.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("with_alpha"),
                    Control::switch("swatches").code(|_, values| match is_on(values, "swatches") {
                        true => vec!["swatches: SWATCHES".to_string()],
                        false => vec![],
                    }),
                    Control::switch("with_picker").default("true").code(|_, values| {
                        match is_on(values, "with_picker") {
                            true => vec![],
                            false => vec!["with_picker: false".to_string()],
                        }
                    }),
                    Control::switch("with_preview").default("true").code(|_, values| {
                        match is_on(values, "with_preview") {
                            true => vec![],
                            false => vec!["with_preview: false".to_string()],
                        }
                    }),
                    Control::switch("with_eye_dropper").default("true").code(|_, values| {
                        match is_on(values, "with_eye_dropper") {
                            true => vec![],
                            false => vec!["with_eye_dropper: false".to_string()],
                        }
                    }),
                    Control::switch("fix_on_blur").default("true").code(|_, values| {
                        match is_on(values, "fix_on_blur") {
                            true => vec![],
                            false => vec!["fix_on_blur: false".to_string()],
                        }
                    }),
                    Control::switch("disallow_input"),
                    Control::switch("close_on_swatch_click"),
                    Control::switch("label").default("true").code(|_, values| {
                        match is_on(values, "label") {
                            true => vec![r#"label: "Brand color""#.to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match is_on(values, "description") {
                            true => vec![r#"description: "Used for buttons and links.""#.to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    ColorFieldDemo { values }
                },
            }
        }
    }
}

/// Its own component so the color survives a control change.
#[component]
fn ColorFieldDemo(values: DemoValues) -> Element {
    let mut color = use_signal(|| ColorCode::hex(0x228be6));

    let swatches = match is_on(&values, "swatches") {
        true => Swatches::from(SWATCHES),
        false => Swatches::default(),
    };

    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%").max_width("320px"),
            ColorField {
                value: color(),
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
                size: values.str("size"),
                radius: values.str("radius"),
                format: values.str("format"),
                with_alpha: is_on(&values, "with_alpha"),
                swatches,
                with_picker: is_on(&values, "with_picker"),
                with_preview: is_on(&values, "with_preview"),
                with_eye_dropper: is_on(&values, "with_eye_dropper"),
                fix_on_blur: is_on(&values, "fix_on_blur"),
                disallow_input: is_on(&values, "disallow_input"),
                close_on_swatch_click: is_on(&values, "close_on_swatch_click"),
                label: is_on(&values, "label").then(|| "Brand color".to_string()),
                description: is_on(&values, "description")
                    .then(|| "Used for buttons and links.".to_string()),
                status: match values.str("status").as_str() {
                    "warning" => FieldStatus::Warning("Low contrast on white.".to_string()),
                    "error" => FieldStatus::Error("Pick a brand color.".to_string()),
                    _ => FieldStatus::Valid,
                },
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
            }
            Text { size: "sm", "value: {color().to_hexa()}" }
        }
    }
}
