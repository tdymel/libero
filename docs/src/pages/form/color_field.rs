use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Code, ColorCode, ColorField, FieldStatus, Flex, Kbd, SliderChangeEvent, Swatches, Text,
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
                        .doc("Strictly controlled. Pair it with `oninput`."),
                    prop("oninput", "EventHandler<SliderChangeEvent<ColorCode>>")
                        .doc("Every new color. A drag in the dropdown sends `Start` and `End` around its moves. A key press, a swatch, typed text that parses and the eyedropper send `Change` then `End`."),
                    prop("validate", "Validators<ColorCode>")
                        .doc("Rules over the color, shown once the field loses focus or its form is submitted."),
                    prop("format", "ColorFormat")
                        .default("hex, or hexa with alpha")
                        .doc("How the text shows the color, and so what the field posts. Typing accepts every form either way."),
                    prop("with_alpha", "bool")
                        .default("false")
                        .doc("Shows the alpha slider in the dropdown. Without it a translucent color arrives opaque."),
                    prop("swatches", "Swatches").doc("Preset colors in the dropdown. Takes `ColorCode`s or CSS strings."),
                    prop("swatches_per_row", "usize").doc("Caps how many swatches share a row. Unset, they wrap to fill the width."),
                    prop("with_picker", "bool")
                        .default("true")
                        .doc("`false` leaves only the swatches in the dropdown, and no dropdown without them."),
                    prop("with_preview", "bool").default("true").doc("Shows the color as a swatch in the leading slot."),
                    prop("with_eye_dropper", "bool")
                        .default("true")
                        .doc("Adds a button in the trailing slot that picks a color off the screen. Shown only where the browser supports it, Chromium today."),
                    prop("disallow_input", "bool")
                        .default("false")
                        .doc("Makes the text read-only, so a color comes from the dropdown alone."),
                    prop("fix_on_blur", "bool")
                        .default("true")
                        .doc("Text that does not parse goes back to the last valid color on blur. Off, it stays and shows `color.invalid` as an error."),
                    prop("close_on_swatch_click", "bool").default("false").doc("Picking a swatch closes the dropdown."),
                    prop("name", "FieldName<ColorCode>")
                        .doc("What the field posts as, the text in `format`. A path such as `Theme::FIELDS.accent()` also binds the color to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("placeholder", "String").doc("Shown while the text is empty."),
                    prop("size", "Size").default("md").doc("Control height, font size and the dropdown's picker."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
                    prop("label", "Caption").doc("The field's caption."),
                    prop("description", "Caption").doc("Between the label and the control."),
                    prop("helper", "Caption").doc("Under the control."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables typing and the dropdown, and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post."),
                ]).extends("input"),
            ],
            lead: rsx! {
                Text {
                    "A text field holding a "
                    Code { source: "ColorCode" }
                    ", with a preview swatch, an eyedropper and a "
                    Code { source: "ColorPicker" }
                    " in a dropdown. The text accepts every form a "
                    Code { source: "ColorCode" }
                    " parses, such as hex, "
                    Code { source: "rgb()" }
                    " or "
                    Code { source: "hsl()" }
                    ". It stays as typed while the field has focus, each parse sends the color, "
                    "and on blur it shows "
                    Code { source: "value" }
                    " in "
                    Code { source: "format" }
                    "."
                }
            },
            // snippet: item const SWATCHES: [&str; 7] = [""; 7];
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
                            false => vec![r#"aria_label: "Brand color""#.to_string()],
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
            DocSection {
                title: "Accessibility",
                Text {
                    "Focus opens the dropdown and stays in the text, so typing works at once. "
                    Kbd { "↓" } " moves focus into the picker, onto the saturation area or the first swatch, where the "
                    Code { source: "ColorPicker" }
                    " keys apply. "
                    Kbd { "Escape" } " goes back to the text, and so does a swatch that closes the dropdown. "
                    Kbd { "Tab" } " past either end of the dropdown leaves the field. "
                    "Focus leaving both the text and the dropdown closes it. A mouse click in the dropdown leaves focus in the text."
                }
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
                aria_label: (!is_on(&values, "label")).then_some("Brand color"),
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
