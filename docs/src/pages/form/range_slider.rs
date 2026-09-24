use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::SliderPart;
use libero::{
    components::{
        Code, FieldStatus, Flex, RangeSlider, SliderChangeEvent, SliderMark, SliderValue, Text,
    },
    sx::sx,
    use_theme,
};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

/// The demo's own discrete type - a range slides over the same values a
/// single-thumb `Slider` does.
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
/// discrete range, so it is part of the example, not a separate section.
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

/// The value type decides `value`, `oninput` and what the thumbs report, so
/// the mode control prints both.
fn mode_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match discrete(values) {
        true => vec![
            "value: quality()".to_string(),
            "oninput: move |event: SliderChangeEvent<(Quality, Quality)>| { quality.set(event.value()); last_quality.set(event) }"
                .to_string(),
        ],
        false => vec![
            "value: price()".to_string(),
            "oninput: move |event: SliderChangeEvent<(f64, f64)>| { price.set(event.value()); last.set(event) }"
                .to_string(),
        ],
    }
}

/// A bound in the value's own type. Quiet at the option the range would pick
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

/// Prints unquoted floats under the real prop names; the controls' `_value` suffix avoids
/// clashing with the discrete `min` and `max`.
fn number_code(control: &Control, values: &DemoValues) -> Vec<String> {
    if discrete(values) {
        return vec![];
    }
    match values.str(control.name).as_str() {
        "auto" | "0.0" => vec![],
        value => vec![format!(
            "{}: {value}",
            control.name.trim_end_matches("_value")
        )],
    }
}

/// Discretely, `min_range` is a count of options rather than a distance.
fn min_range_code(control: &Control, values: &DemoValues) -> Vec<String> {
    if continuous(values) {
        return vec![];
    }
    match values.str(control.name).as_str() {
        "0" => vec![],
        stride => vec![format!("min_range: {stride}usize")],
    }
}

fn number(values: &DemoValues, name: &str) -> Option<f64> {
    values.str(name).parse::<f64>().ok()
}

/// Ticks at both ends and the midpoint of the *current* range - a mark at 50
/// means nothing on a 0-to-10 slider.
fn mark_values(values: &DemoValues) -> [f64; 3] {
    let min = values.str("min_value").parse().unwrap_or(0.0);
    let max = values.str("max_value").parse().unwrap_or(100.0);
    [min, (min + max) / 2.0, max]
}

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
            vec![r#"format: Callback::new(|value: f64| format!("{value} EUR"))"#.to_string()]
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

/// The readout, the other half of a controlled range, shows the thumbs never cross. Printed too.
fn wrap_readout(values: &DemoValues, code: &str) -> String {
    let (declaration, signal, last) = match discrete(values) {
        true => (QUALITY, "quality():?", "last_quality()"),
        false => ("", "price():?", "last()"),
    };
    format!(
        "{declaration}Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    sx: sx().width(\"100%\"),\n{}    Text {{ size: \"sm\", \"value: {{{signal}}} - last event: {{{last}:?}}\" }}\n}}",
        indent(code)
    )
}

#[component]
pub fn RangeSliderPage() -> Element {
    let theme = use_theme();
    let mut price = use_signal(|| (20.0, 80.0));
    let mut quality = use_signal(|| (Quality::Low, Quality::High));
    let mut last = use_signal(|| SliderChangeEvent::Change((20.0, 80.0)));
    let mut last_quality = use_signal(|| SliderChangeEvent::Change((Quality::Low, Quality::High)));

    rsx! {
        DocPage {
            title: "RangeSlider",
            source: "libero/src/components/form/slider",
            markdown: "/md/range_slider.md",
            properties: vec![
                props("RangeSlider", vec![
                    prop("size", "Size").default("md").doc("Track, thumb and font size."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Accent color. A theme color name or any CSS color."),
                    prop("value", "Option<(V, V)>")
                        .doc("The two ends, in track order. Pair it with `oninput`, or bind it with a path `name` inside a `Form`."),
                    prop("oninput", "EventHandler<SliderChangeEvent<(V, V)>>")
                        .doc("Fires per value while dragging. `Start` and `End` bracket a drag, `Change` carries each new pair. A key press sends `Change`, then `End`."),
                    prop("min", "V")
                        .default("first option, or 0.0")
                        .doc("Lower bound of the track, in the value's own type."),
                    prop("max", "V")
                        .default("last option, or 100.0")
                        .doc("Upper bound of the track, in the value's own type."),
                    prop("step", "V::Step")
                        .doc("How far one step goes from `min`. A count of options on a discrete scale, a value on a continuous one. Also sets how many decimals a value keeps."),
                    prop("min_range", "V::Step")
                        .default("0")
                        .doc("The smallest gap the thumbs keep, in the unit of `step`. At 0 they may meet, and they never cross."),
                    prop("format", "Callback<V, String>")
                        .default("bare value, or SliderValue::label")
                        .doc("Text of the bubbles and each thumb's `aria-valuetext`. On a discrete scale it also names the marks, so this is where a translation goes."),
                    prop("marks", "Vec<SliderMark<V>>")
                        .default("one per option, discretely")
                        .doc("Ticks on the track. A labeled one gets a caption below it. Replaces the marks a discrete scale draws itself."),
                    prop("aria_label_from", "String")
                        .default("slider.minimum")
                        .doc("Names the lower thumb. Unset, the localization's `slider.minimum`, \"Minimum\" in English."),
                    prop("aria_label_to", "String")
                        .default("slider.maximum")
                        .doc("Names the upper thumb. Unset, the localization's `slider.maximum`, \"Maximum\" in English."),
                    prop("name", "FieldName<(V, V)>")
                        .doc("Posts the pair as two hidden inputs of that name, in track order. A path such as `Settings::FIELDS.price()` also binds the pair to the surrounding `Form`'s value when there is no `oninput`."),
                    prop("validate", "Validators<(V, V)>")
                        .doc("Rules over the pair, shown once the slider loses focus or its form is submitted."),
                    prop("label", "Caption")
                        .doc("The caption above the track. Both thumbs' names start with it."),
                    prop("description", "Caption")
                        .doc("Between the label and the track. What the range means."),
                    prop("helper", "Caption")
                        .doc("Under the track, below the mark captions."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Adds an asterisk to the label. No `aria-required`: ARIA does not allow it on a slider, which always holds a value."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Takes the thumbs out of the tab order and dims the slider."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable. `disabled` drops the slider from the tab order and the post instead."),
                ])
                .parts("SliderPart", vec![
                    (SliderPart::Label, "The label above the control."),
                    (SliderPart::Required, "The required asterisk, in the label."),
                    (SliderPart::Description, "The caption between the label and the control."),
                    (SliderPart::Control, "The slider under the label: the track and the room around it."),
                    (SliderPart::Track, "The rail the thumbs run along."),
                    (SliderPart::Bar, "The filled stretch of the track."),
                    (SliderPart::Mark, "One tick on the track."),
                    (SliderPart::MarkLabel, "A tick's caption."),
                    (SliderPart::Thumb, "The handle; a range has two."),
                    (SliderPart::Helper, "The caption under the control."),
                    (SliderPart::Status, "The validation message."),
                ]),
                props("SliderMark", vec![
                    prop("value", "V").doc("Where the tick sits on the track."),
                    prop("label", "String").doc("Caption below the tick. Leave it out for a bare tick."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Left", "Right", "Up", "Down"], "Move the focused thumb one step.")
                .key(["Shift+Arrow", "PageUp", "PageDown"], "Move the focused thumb `big_step` steps.")
                .key(["Home", "End"], "Move the focused thumb to the end, stopping at the other thumb.")
                .handles([
                    "The two thumbs sit in a `role=\"group\"` named by the label, and each is its own `role=\"slider\"`, as in the ARIA multi-thumb slider pattern. A single `Slider` is one slider and needs no group.",
                    "Each thumb is named by the label plus its own word, such as \"Price Minimum\" and \"Price Maximum\", from the localization's `slider.minimum` and `slider.maximum`.",
                    "On a discrete range the mark captions are hidden from screen readers, since the thumbs already name each value.",
                    "A `role=\"slider\"` takes no `aria-required`, so a `required` range says the localization's `slider.required` word in each thumb's name instead, such as \"Price required Minimum\". The asterisk stays hidden from screen readers.",
                ])
                .must([
                    "Set `aria_label_from` and `aria_label_to` when those words do not fit.",
                    "Set `format` when a bare number does not say the unit. To translate a discrete range, pass `format`, as on a `Slider`.",
                ]),
            lead: rsx! {
                Text {
                    "Two thumbs on one track, for a span instead of a point. It takes the same "
                    "values as "
                    Code { source: "Slider" }
                    ", as a pair in track order. The thumbs never cross, and each stops at the "
                    "other or "
                    Code { source: "min_range" }
                    " short of it."
                }
            },
            // snippet: let mut price = use_signal(|| (20.0, 80.0));
            // snippet: let mut quality = use_signal(|| (Quality::Low, Quality::High));
            // snippet: let mut last = use_signal(|| SliderChangeEvent::Change((20.0, 80.0)));
            // snippet: let mut last_quality = use_signal(|| SliderChangeEvent::Change((Quality::Low, Quality::High)));
            Demo {
                component: "RangeSlider",
                children_text: "",
                controls: vec![
                    Control::toggle("mode", ["discrete", "continuous"])
                        .default("continuous")
                        .labels(["Discrete", "Continuous"])
                        .code(mode_code),
                    Control::slider("size", SIZES).default(theme.slider.size.as_str()),
                    Control::color("color"),
                    Control::slider("min", Quality::ALL)
                        .code(bound_code)
                        .hidden_when(continuous),
                    Control::slider("max", Quality::ALL)
                        .default("ultra")
                        .code(bound_code)
                        .hidden_when(continuous),
                    Control::slider("min_range", ["0", "1", "2"])
                        .code(min_range_code)
                        .hidden_when(continuous),
                    Control::slider("min_value", ["auto", "0.0", "10.0", "50.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("max_value", ["auto", "50.0", "100.0", "200.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("step_value", ["auto", "0.5", "5.0", "10.0", "25.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("min_range_value", ["0.0", "10.0", "25.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("A wide range costs more.".into())"#
                                    .to_string(),
                            ],
                            "error" => vec![r#"status: "Pick a narrower range.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("marks").code(marks_code),
                    Control::switch("format").code(format_code),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec![r#"label: "Price""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec![r#"description: "What you are willing to pay.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec![r#"helper: "Both ends included.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| {
                    let status = match values.str("status").as_str() {
                        "warning" => FieldStatus::Warning("A wide range costs more.".to_string()),
                        "error" => FieldStatus::Error("Pick a narrower range.".to_string()),
                        _ => FieldStatus::Valid,
                    };
                    rsx! {
                        Flex {
                            direction: "column",
                            gap: "sm",
                            sx: sx().width("100%"),
                            if discrete(&values) {
                                RangeSlider {
                                    value: quality(),
                                    size: values.str("size"),
                                    color: values.str("color"),
                                    min: quality_of(&values, "min"),
                                    max: quality_of(&values, "max"),
                                    min_range: values.str("min_range").parse::<usize>().unwrap_or(0),
                                    marks: discrete_marks(&values),
                                    format: is_on(&values, "format")
                                        .then(|| {
                                            Callback::new(|quality: Quality| {
                                                format!("{quality:?} quality")
                                            })
                                        }),
                                    label: is_on(&values, "label").then(|| "Price".to_string()),
                                    description: is_on(&values, "description")
                                        .then(|| "What you are willing to pay.".to_string()),
                                    helper: is_on(&values, "helper")
                                        .then(|| "Both ends included.".to_string()),
                                    status: status.clone(),
                                    required: is_on(&values, "required").then_some(true),
                                    disabled: is_on(&values, "disabled").then_some(true),
                                    oninput: move |event: SliderChangeEvent<(Quality, Quality)>| {
                                        quality.set(event.value());
                                        last_quality.set(event)
                                    },
                                }
                                Text {
                                    size: "sm",
                                    "value: {quality():?} - last event: {last_quality():?}"
                                }
                            } else {
                                RangeSlider {
                                    value: price(),
                                    size: values.str("size"),
                                    color: values.str("color"),
                                    min: number(&values, "min_value"),
                                    max: number(&values, "max_value"),
                                    step: number(&values, "step_value"),
                                    min_range: number(&values, "min_range_value"),
                                    marks: continuous_marks(&values),
                                    format: is_on(&values, "format")
                                        .then(|| Callback::new(|value: f64| format!("{value} EUR"))),
                                    label: is_on(&values, "label").then(|| "Price".to_string()),
                                    description: is_on(&values, "description")
                                        .then(|| "What you are willing to pay.".to_string()),
                                    helper: is_on(&values, "helper")
                                        .then(|| "Both ends included.".to_string()),
                                    status,
                                    required: is_on(&values, "required").then_some(true),
                                    disabled: is_on(&values, "disabled").then_some(true),
                                    oninput: move |event: SliderChangeEvent<(f64, f64)>| {
                                        price.set(event.value());
                                        last.set(event)
                                    },
                                }
                                Text { size: "sm", "value: {price():?} - last event: {last():?}" }
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_readout),
            }
        }
    }
}
