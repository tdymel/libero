use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent};
use dioxus::prelude::*;
use libero::{
    components::{Code, CodeBlock, Flex, Slider, SliderChangeEvent, SliderMark, Text},
    sx::sx,
    theme::Size,
    use_theme,
};

const SIZES: [&str; 5] = ["xs", "sm", "md", "lg", "xl"];

/// Every `Size`, since that is what a discrete slider over one shows.
const VALUES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

const SLIDER_VALUE: &str = r#"#[derive(Clone, Copy, PartialEq)]
enum Quality { Low, Medium, High }

impl SliderValue for Quality {
    type Step = usize;

    fn options() -> Option<&'static [Self]> {
        Some(&[Self::Low, Self::Medium, Self::High])
    }

    fn label(&self) -> String {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
        .to_string()
    }
}

// `min`/`max` are written in the value's own type; `step` counts options,
// so `step: 1.5` is a compile error rather than a surprise at runtime.
rsx! {
    Slider {
        value: quality(),
        min: Quality::Low,
        step: 1,
        on_change: move |event: SliderChangeEvent<Quality>| quality.set(event.value()),
    }
}"#;

fn discrete(values: &DemoValues) -> bool {
    values.str("mode") == "discrete"
}

fn continuous(values: &DemoValues) -> bool {
    !discrete(values)
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

/// The value type decides `value`, `on_change` and the name the thumb reports,
/// so the mode control prints all three.
fn mode_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match discrete(values) {
        true => vec![
            r#"aria_label: "Size""#.to_string(),
            "value: size()".to_string(),
            "on_change: move |event: SliderChangeEvent<Size>| { size.set(event.value()); last_size.set(event) }"
                .to_string(),
        ],
        false => vec![
            r#"aria_label: "Volume""#.to_string(),
            "value: volume()".to_string(),
            "on_change: move |event: SliderChangeEvent| { volume.set(event.value()); last.set(event) }"
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
            "{}: Size::{}{}",
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
            r#"marks: vec![SliderMark::labeled(Size::Xs, "small"), SliderMark::labeled(Size::Xxl, "large")]"#
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

fn label_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match (is_on(values, "label"), discrete(values)) {
        (false, _) => vec![],
        (true, true) => {
            vec![r#"label: Callback::new(|size: Size| format!("{size:?} size"))"#.to_string()]
        }
        (true, false) => {
            vec![r#"label: Callback::new(|value: f64| format!("{value}%"))"#.to_string()]
        }
    }
}

fn size_of(values: &DemoValues, name: &str) -> Size {
    Size::from(values.str(name).as_str())
}

fn discrete_marks(values: &DemoValues) -> Vec<SliderMark<Size>> {
    match is_on(values, "marks") {
        true => vec![
            SliderMark::labeled(Size::Xs, "small"),
            SliderMark::labeled(Size::Xxl, "large"),
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
    let (signal, last) = match discrete(values) {
        true => ("size():?", "last_size()"),
        false => ("volume()", "last()"),
    };
    format!(
        "Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\"),\n{}    Text {{ size: \"sm\", \"value: {{{signal}}} - last event: {{{last}:?}}\" }}\n}}",
        indent(code)
    )
}

#[component]
pub fn SliderPage() -> Element {
    let theme = use_theme();
    let mut volume = use_signal(|| 40.0);
    let mut size = use_signal(|| Size::Md);
    let mut last = use_signal(|| SliderChangeEvent::Change(40.0));
    let mut last_size = use_signal(|| SliderChangeEvent::Change(Size::Md));

    rsx! {
        DocPage {
            title: "Slider",
            lead: rsx! {
                Text {
                    "A value dragged along a track. Controlled: it renders "
                    Code { source: "value" }
                    " and asks for a new one through "
                    Code { source: "on_change" }
                    ". Pointer, touch and keyboard all drive it - the thumb is a "
                    Code { source: "role=\"slider\"" }
                    " with arrows, Page keys, Home and End."
                }
            },
            DocSection {
                title: "Usage",
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
                        Control::slider("min", VALUES).code(bound_code).hidden_when(continuous),
                        Control::slider("max", VALUES)
                            .default("xxl")
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
                        Control::switch("marks").code(marks_code),
                        Control::switch("label").code(label_code),
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
                                        aria_label: "Size",
                                        value: size(),
                                        size: values.str("size"),
                                        radius: values.str("radius"),
                                        color: values.str("color"),
                                        min: size_of(&values, "min"),
                                        max: size_of(&values, "max"),
                                        step: values.str("step").parse::<usize>().unwrap_or(1),
                                        marks: discrete_marks(&values),
                                        label: is_on(&values, "label")
                                            .then(|| Callback::new(|size: Size| format!("{size:?} size"))),
                                        disabled: is_on(&values, "disabled").then_some(true),
                                        on_change: move |event: SliderChangeEvent<Size>| {
                                            size.set(event.value());
                                            last_size.set(event)
                                        },
                                    }
                                    Text { size: "sm", "value: {size():?} - last event: {last_size():?}" }
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
                                        label: is_on(&values, "label")
                                            .then(|| Callback::new(|value: f64| format!("{value}%"))),
                                        disabled: is_on(&values, "disabled").then_some(true),
                                        on_change: move |event: SliderChangeEvent| {
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
            DocSection {
                title: "Discrete values",
                Text {
                    "A value type that lists its options makes the slider discrete. "
                    Code { source: "SliderValue" }
                    " is what the component slides over - libero implements it for "
                    Code { source: "f64" }
                    " (continuous) and for "
                    Code { source: "Size" }
                    ", and an ordered enum of your own needs the two methods it "
                    "cannot guess. The value goes in and comes back as that type; "
                    "nothing indexes an array."
                }
                CodeBlock { source: SLIDER_VALUE, language: "rust" }
            }
        }
    }
}
