use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Code, FieldStatus, Flex, Slider, SliderChangeEvent, SliderMark, SliderValue, Text,
    },
    sx::sx,
    use_theme,
};

const SIZES: [&str; 5] = ["xs", "sm", "md", "lg", "xl"];

/// The demo's own discrete type, so the code block can show the derive that
/// makes it one - the preview slides over exactly this enum.
#[derive(Clone, Copy, Debug, PartialEq, SliderValue)]
enum Quality {
    Low,
    Medium,
    High,
    #[slider(label = "Max")]
    Ultra,
}

impl Quality {
    const ALL: [&'static str; 4] = ["low", "medium", "high", "ultra"];

    fn parse(value: &str) -> Self {
        match value {
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            _ => Self::Ultra,
        }
    }
}

/// Printed above the rsx in discrete mode: without the derive there is no
/// discrete slider, so it is part of the example, not a separate section.
const QUALITY: &str = r#"#[derive(Clone, Copy, Debug, PartialEq, SliderValue)]
enum Quality {
    Low,
    Medium,
    High,
    #[slider(label = "Max")]
    Ultra,
}

"#;

fn discrete(values: &DemoValues) -> bool {
    values.str("mode") == "discrete"
}

fn continuous(values: &DemoValues) -> bool {
    !discrete(values)
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

/// The value type decides `value`, `oninput` and the name the thumb reports,
/// so the mode control prints all three.
fn mode_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match discrete(values) {
        true => vec![
            r#"aria_label: "Quality""#.to_string(),
            "value: quality()".to_string(),
            "oninput: move |event: SliderChangeEvent<Quality>| { quality.set(event.value()); last_quality.set(event) }"
                .to_string(),
        ],
        false => vec![
            r#"aria_label: "Volume""#.to_string(),
            "value: volume()".to_string(),
            "oninput: move |event: SliderChangeEvent| { volume.set(event.value()); last.set(event) }"
                .to_string(),
        ],
    }
}

/// A bound in the value's own type. Quiet at the option the slider would pick
/// anyway - the first for `min`, the last for `max`.
fn bound_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    match continuous(values) || value == control.default {
        true => vec![],
        false => vec![format!(
            "{}: Quality::{}{}",
            control.name,
            value[..1].to_uppercase(),
            &value[1..]
        )],
    }
}

/// A stride over the options - `1` is every one of them, which is the default.
fn stride_code(control: &Control, values: &DemoValues) -> Vec<String> {
    if continuous(values) {
        return vec![];
    }
    match values.str(control.name).as_str() {
        "1" => vec![],
        stride => vec![format!("step: {stride}")],
    }
}

/// `min`/`max`/`step` print as unquoted floats, under their real prop names -
/// the control is `min_value` only because the discrete `min` owns that name.
fn number_code(control: &Control, values: &DemoValues) -> Vec<String> {
    if discrete(values) {
        return vec![];
    }
    match values.str(control.name).as_str() {
        "auto" => vec![],
        value => vec![format!(
            "{}: {value}",
            control.name.trim_end_matches("_value")
        )],
    }
}

fn number(values: &DemoValues, name: &str) -> Option<f64> {
    values.str(name).parse::<f64>().ok()
}

/// Ticks at both ends and the midpoint of the *current* range - a mark at 50
/// means nothing on a 0.5-to-3 slider.
fn mark_values(values: &DemoValues) -> [f64; 3] {
    let min = values.str("min_value").parse().unwrap_or(0.0);
    let max = values.str("max_value").parse().unwrap_or(100.0);
    [min, (min + max) / 2.0, max]
}

/// Discretely, `marks` *replaces* the one-per-option set the type derives;
/// continuously there is nothing to derive, so it adds them.
fn marks_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    if !is_on(values, "marks") {
        return vec![];
    }
    match discrete(values) {
        true => vec![
            r#"marks: vec![SliderMark::labeled(Quality::Low, "cheap"), SliderMark::labeled(Quality::Ultra, "pricey")]"#
                .to_string(),
        ],
        false => {
            let [min, mid, max] = mark_values(values);
            vec![format!(
                "marks: vec![SliderMark::labeled({min:?}, \"{min}\"), SliderMark::new({mid:?}), SliderMark::labeled({max:?}, \"{max}\")]"
            )]
        }
    }
}

fn format_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match (is_on(values, "format"), discrete(values)) {
        (false, _) => vec![],
        (true, true) => {
            vec![
                r#"format: Callback::new(|quality: Quality| format!("{quality:?} quality"))"#
                    .to_string(),
            ]
        }
        (true, false) => {
            vec![r#"format: Callback::new(|value: f64| format!("{value}%"))"#.to_string()]
        }
    }
}

fn quality_of(values: &DemoValues, name: &str) -> Quality {
    Quality::parse(&values.str(name))
}

fn discrete_marks(values: &DemoValues) -> Vec<SliderMark<Quality>> {
    match is_on(values, "marks") {
        true => vec![
            SliderMark::labeled(Quality::Low, "cheap"),
            SliderMark::labeled(Quality::Ultra, "pricey"),
        ],
        false => Vec::new(),
    }
}

fn continuous_marks(values: &DemoValues) -> Vec<SliderMark> {
    match is_on(values, "marks") {
        true => {
            let [min, mid, max] = mark_values(values);
            vec![
                SliderMark::labeled(min, min.to_string()),
                SliderMark::new(mid),
                SliderMark::labeled(max, max.to_string()),
            ]
        }
        false => Vec::new(),
    }
}

/// The readout is the other half of a controlled slider - `Start`/`End`
/// bracket one drag, `Change` carries every value in between - so it belongs
/// in the preview, and the code block prints it.
fn wrap_readout(values: &DemoValues, code: &str) -> String {
    let (declaration, signal, last) = match discrete(values) {
        true => (QUALITY, "quality():?", "last_quality()"),
        false => ("", "volume()", "last()"),
    };
    format!(
        "{declaration}Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\"),\n{}    Text {{ size: \"sm\", \"value: {{{signal}}} - last event: {{{last}:?}}\" }}\n}}",
        indent(code)
    )
}

#[component]
pub fn SliderPage() -> Element {
    let theme = use_theme();
    let mut volume = use_signal(|| 40.0);
    let mut quality = use_signal(|| Quality::Medium);
    let mut last = use_signal(|| SliderChangeEvent::Change(40.0));
    let mut last_quality = use_signal(|| SliderChangeEvent::Change(Quality::Medium));

    rsx! {
        DocPage {
            title: "Slider",
            source: "libero/src/components/form/slider",
            markdown: "/md/slider.md",
            properties: vec![
                props("Slider", vec![
                    prop("value", "V").doc("Strictly controlled - pair it with `oninput`."),
                    prop("min", "V")
                        .default("first option, or 0.0")
                        .doc("Lower bound, written in the value's own type."),
                    prop("max", "V")
                        .default("last option, or 100.0")
                        .doc("Upper bound, written in the value's own type."),
                    prop("step", "V::Step")
                        .doc("Distance one step covers, measured from `min`: a count of options discretely, a value continuously. Also sets how many decimals an emitted value keeps."),
                    prop("size", "Size").default("md").doc("Controls track, thumb, and font size."),
                    prop("radius", "Size")
                        .default("xl")
                        .doc("Track corner radius; the thumb is always a circle."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Accent color; a theme color name or a literal CSS color."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the slider."),
                    prop("format", "Callback<V, String>")
                        .default("bare value, or SliderValue::label")
                        .doc("Formats the bubble shown on hover, drag and keyboard focus, and sets the thumb's `aria-valuetext`."),
                    prop("marks", "Vec<SliderMark<V>>")
                        .default("one per option, discretely")
                        .doc("Ticks on the track; a labeled one gets a caption below it. Replaces the marks a discrete scale derives."),
                    prop("aria_label", "String")
                        .doc("Names the thumb when the field has no `label` - an `aria_label` in `attributes` would land on the wrapper instead."),
                    prop("name", "String")
                        .doc("Emits a hidden input of that name, so the value posts with a form."),
                    prop("oninput", "EventHandler<SliderChangeEvent<V>>")
                        .doc("Fires per value - a drag is the DOM's `input` event. `Start`/`End` bracket a drag, `Change` carries every new value."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the track. Named by `aria-labelledby`, since `for` cannot name the thumb."),
                    prop("description", "Caption")
                        .doc("Between the label and the track: what the value means."),
                    prop("helper", "Caption")
                        .doc("Under the track, below the mark captions."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Adds `aria-required` to the thumb and marks the label."),
                ]),
                props("SliderMark", vec![
                    prop("value", "V").doc("Where the tick sits on the track."),
                    prop("label", "String").doc("Caption shown below the tick; omit for an unlabeled mark."),
                ])
                .without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "A value dragged along a track. Controlled: it renders "
                    Code { source: "value" }
                    " and asks for a new one through "
                    Code { source: "oninput" }
                    ". Pointer, touch and keyboard all drive it - the thumb is a "
                    Code { source: "role=\"slider\"" }
                    " with arrows, Page keys, Home and End."
                }
                Text {
                    "What it slides over is a "
                    Code { source: "SliderValue" }
                    ": libero implements it for "
                    Code { source: "f64" }
                    " - a continuous range - and an ordered enum of your own derives "
                    "it. A type that lists its options makes the slider discrete, and "
                    "the range, step grid, marks and captions all come from that list."
                }
            },
            Demo {
                component: "Slider",
                children_text: "",
                controls: vec![
                    // The value's type is the mode: an ordered enum makes
                    // the slider discrete, `f64` leaves it continuous. Each
                    // brings its own props, so the set below swaps with it.
                    Control::toggle("mode", ["discrete", "continuous"])
                        .labels(["Discrete", "Continuous"])
                        .code(mode_code),
                    Control::slider("size", SIZES).default(theme.slider.size.as_str()),
                    Control::slider("radius", SIZES).default(theme.slider.radius.as_str()),
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    ),
                    Control::slider("min", Quality::ALL)
                        .code(bound_code)
                        .hidden_when(continuous),
                    Control::slider("max", Quality::ALL)
                        .default("ultra")
                        .code(bound_code)
                        .hidden_when(continuous),
                    Control::slider("step", ["1", "2", "3"])
                        .code(stride_code)
                        .hidden_when(continuous),
                    Control::slider("min_value", ["auto", "0.5", "10.0", "50.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("max_value", ["auto", "3.0", "50.0", "200.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("step_value", ["auto", "0.1", "5.0", "10.0", "25.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("Above the free tier.".into())"#
                                    .to_string(),
                            ],
                            "error" => vec![r#"status: "Pick a lower setting.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("marks").code(marks_code),
                    Control::switch("format").code(format_code),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec![r#"label: "Quality""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec![r#"description: "What the encoder aims for.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec![r#"helper: "Higher costs more.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| {
                    rsx! {
                        Flex {
                            direction: "column",
                            gap: "sm",
                            sx: sx().width("100%"),
                            if discrete(&values) {
                                Slider {
                                    aria_label: "Quality",
                                    value: quality(),
                                    size: values.str("size"),
                                    radius: values.str("radius"),
                                    color: values.str("color"),
                                    min: quality_of(&values, "min"),
                                    max: quality_of(&values, "max"),
                                    step: values.str("step").parse::<usize>().unwrap_or(1),
                                    marks: discrete_marks(&values),
                                    format: is_on(&values, "format")
                                        .then(|| {
                                            Callback::new(|quality: Quality| {
                                                format!("{quality:?} quality")
                                            })
                                        }),
                                    label: is_on(&values, "label")
                                        .then(|| "Quality".to_string()),
                                    description: is_on(&values, "description")
                                        .then(|| "What the encoder aims for.".to_string()),
                                    helper: is_on(&values, "helper")
                                        .then(|| "Higher costs more.".to_string()),
                                    status: match values.str("status").as_str() {
                                        "warning" => {
                                            FieldStatus::Warning("Above the free tier.".to_string())
                                        }
                                        "error" => {
                                            FieldStatus::Error("Pick a lower setting.".to_string())
                                        }
                                        _ => FieldStatus::Valid,
                                    },
                                    required: is_on(&values, "required").then_some(true),
                                    disabled: is_on(&values, "disabled").then_some(true),
                                    oninput: move |event: SliderChangeEvent<Quality>| {
                                        quality.set(event.value());
                                        last_quality.set(event)
                                    },
                                }
                                Text {
                                    size: "sm",
                                    "value: {quality():?} - last event: {last_quality():?}"
                                }
                            } else {
                                Slider {
                                    aria_label: "Volume",
                                    value: volume(),
                                    size: values.str("size"),
                                    radius: values.str("radius"),
                                    color: values.str("color"),
                                    min: number(&values, "min_value"),
                                    max: number(&values, "max_value"),
                                    step: number(&values, "step_value"),
                                    marks: continuous_marks(&values),
                                    format: is_on(&values, "format")
                                        .then(|| Callback::new(|value: f64| format!("{value}%"))),
                                    label: is_on(&values, "label")
                                        .then(|| "Volume".to_string()),
                                    description: is_on(&values, "description")
                                        .then(|| "What the encoder aims for.".to_string()),
                                    helper: is_on(&values, "helper")
                                        .then(|| "Higher costs more.".to_string()),
                                    status: match values.str("status").as_str() {
                                        "warning" => {
                                            FieldStatus::Warning("Above the free tier.".to_string())
                                        }
                                        "error" => {
                                            FieldStatus::Error("Pick a lower setting.".to_string())
                                        }
                                        _ => FieldStatus::Valid,
                                    },
                                    required: is_on(&values, "required").then_some(true),
                                    disabled: is_on(&values, "disabled").then_some(true),
                                    oninput: move |event: SliderChangeEvent| {
                                        volume.set(event.value());
                                        last.set(event)
                                    },
                                }
                                Text { size: "sm", "value: {volume()} - last event: {last():?}" }
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_readout),
            }
        }
    }
}
