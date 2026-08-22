use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent};
use dioxus::prelude::*;
use libero::{
    components::{Code, CodeBlock, Flex, Slider, SliderChangeEvent, SliderMark, Text},
    sx::sx,
    theme::Size,
    use_theme,
};

const SIZES: [&str; 5] = ["xs", "sm", "md", "lg", "xl"];

const LABEL: &str = r#"label: Callback::new(|value: f64| format!("{value}%"))"#;

/// Everything else a discrete slider needs - the range, the step grid, one
/// mark per option and every caption - comes from `SliderValue`.
const DISCRETE: [&str; 2] = [
    "value: size()",
    "on_change: move |event: SliderChangeEvent<Size>| { size.set(event.value()); last_size.set(event) }",
];

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
    values.str("discrete") == "true"
}

/// `min`/`max`/`step` all print as unquoted floats, and `auto` is the
/// component's own default. Discrete mode pins all three, so they go quiet.
fn number_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match (discrete(values), values.str(control.name).as_str()) {
        (true, _) | (_, "auto") => vec![],
        (_, value) => vec![format!("{}: {value}", control.name)],
    }
}

fn number(values: &DemoValues, name: &str) -> Option<f64> {
    values.str(name).parse::<f64>().ok()
}

/// Ticks at both ends and the midpoint of the *current* range - a mark at 50
/// means nothing on a 0.5-to-3 slider.
fn mark_values(values: &DemoValues) -> [f64; 3] {
    let min = values.str("min").parse().unwrap_or(0.0);
    let max = values.str("max").parse().unwrap_or(100.0);
    [min, (min + max) / 2.0, max]
}

fn marks(values: &DemoValues) -> Vec<SliderMark> {
    match discrete(values) {
        true => Vec::new(),
        false if values.str("marks") == "true" => {
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
                        // Drives `value` and `on_change` too: a picker over a
                        // list needs its own index signal, not the percentage.
                        Control::switch("discrete").code(|_, values| match discrete(values) {
                            true => [r#"aria_label: "Size""#]
                                .iter()
                                .chain(DISCRETE.iter())
                                .map(|line| line.to_string())
                                .collect(),
                            false => vec![
                                r#"aria_label: "Volume""#.to_string(),
                                "value: volume()".to_string(),
                                "on_change: move |event: SliderChangeEvent| { volume.set(event.value()); last.set(event) }"
                                    .to_string(),
                            ],
                        }),
                        Control::slider("size", SIZES).default(theme.slider.size.as_str()),
                        Control::slider("radius", SIZES).default(theme.slider.radius.as_str()),
                        Control::color(
                            "color",
                            ["primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        Control::slider("min", ["auto", "0.5", "10.0", "50.0"])
                            .code(number_code)
                            .inert_when(discrete),
                        Control::slider("max", ["auto", "3.0", "50.0", "200.0"])
                            .code(number_code)
                            .inert_when(discrete),
                        Control::slider("step", ["auto", "0.1", "5.0", "10.0", "25.0"])
                            .code(number_code)
                            .inert_when(discrete),
                        // The ticks follow the range, so the printed vec is
                        // built from the current `min`/`max`.
                        Control::switch("marks").code(|_, values| {
                            match !discrete(values) && values.str("marks") == "true" {
                                false => vec![],
                                true => {
                                    let [min, mid, max] = mark_values(values);
                                    vec![format!(
                                        "marks: vec![SliderMark::labeled({min:?}, \"{min}\"), SliderMark::new({mid:?}), SliderMark::labeled({max:?}, \"{max}\")]"
                                    )]
                                }
                            }
                        })
                        .inert_when(discrete),
                        Control::switch("label").code(|_, values| {
                            match !discrete(values) && values.str("label") == "true" {
                                true => vec![LABEL.to_string()],
                                false => vec![],
                            }
                        })
                        .inert_when(discrete),
                        Control::switch("disabled"),
                    ],
                    render: move |values: DemoValues| {
                        let discrete = discrete(&values);
                        rsx! {
                            Flex {
                                direction: "column",
                                gap: "sm",
                                sx: sx().width("100%"),
                                if discrete {
                                    Slider {
                                        aria_label: "Size",
                                        value: size(),
                                        size: values.str("size"),
                                        radius: values.str("radius"),
                                        color: values.str("color"),
                                        disabled: (values.str("disabled") == "true").then_some(true),
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
                                        min: number(&values, "min"),
                                        max: number(&values, "max"),
                                        step: number(&values, "step"),
                                        marks: marks(&values),
                                        label: (values.str("label") == "true")
                                            .then(|| Callback::new(|value: f64| format!("{value}%"))),
                                        disabled: (values.str("disabled") == "true").then_some(true),
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
