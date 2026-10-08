use super::slider_demo::{
    Quality, bound_code, continuous, continuous_marks, discrete, discrete_marks, is_on,
    mark_values, marks_code, max_options, min_options, number, number_code, quality_code,
    quality_of, within,
};
use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::components::SliderPart;
use libero::{
    components::{
        Code, FieldStatus, Flex, Slider, SliderChangeEvent, SliderSegment, SliderTrack, Text,
    },
    sx::sx,
    use_theme,
};

const MIN_VALUES: [&str; 4] = ["auto", "0.5", "10.0", "50.0"];
const MAX_VALUES: [&str; 4] = ["auto", "3.0", "50.0", "200.0"];
/// The `bars` switch's heights: a voice message's waveform, as `Audio` draws it.
const WAVEFORM: [f64; 24] = [
    0.3, 0.5, 0.8, 0.6, 0.4, 0.7, 1.0, 0.8, 0.5, 0.3, 0.6, 0.9, 0.7, 0.4, 0.5, 0.8, 0.6, 0.3, 0.4,
    0.7, 0.5, 0.3, 0.4, 0.2,
];

fn track(values: &DemoValues) -> SliderTrack {
    match is_on(values, "bars") {
        true => SliderTrack::Bars(WAVEFORM.to_vec()),
        false => SliderTrack::Line,
    }
}

fn bars_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match is_on(values, "bars") {
        true => vec![format!("track: SliderTrack::Bars(vec!{WAVEFORM:?})")],
        false => vec![],
    }
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

/// The continuous twins of `min_options` and `max_options`; `auto` is 0 and 100.
fn min_value_options(values: &DemoValues) -> Vec<String> {
    let max = number(values, "max_value").unwrap_or(100.0);
    within(&MIN_VALUES, |min| min < max)
}

fn max_value_options(values: &DemoValues) -> Vec<String> {
    let min = number(values, "min_value").unwrap_or(0.0);
    within(&MAX_VALUES, |max| max > min)
}

/// On by default, so the gaps show; a `Bars` track draws no segments, so they go quiet there.
fn segments_on(values: &DemoValues) -> bool {
    is_on(values, "segments") && !is_on(values, "bars")
}

/// "Loud" from 70% of the *current* range, as `mark_values` follows it too.
fn loud_from(values: &DemoValues) -> f64 {
    let [min, _, max] = mark_values(values);
    // Not `min + 0.7 * span`, whose float tail would print in the code.
    (min * 3.0 + max * 7.0) / 10.0
}

fn segments_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    if !segments_on(values) {
        return vec![];
    }
    match discrete(values) {
        true => vec![
            r#"segments: vec![SliderSegment::labeled(Quality::Low, "Free"), SliderSegment::labeled(Quality::High, "Paid")]"#
                .to_string(),
        ],
        false => {
            let [min, _, _] = mark_values(values);
            vec![format!(
                "segments: vec![SliderSegment::labeled({min:?}, \"Comfortable\"), SliderSegment::labeled({:?}, \"Loud\")]",
                loud_from(values)
            )]
        }
    }
}

fn discrete_segments(values: &DemoValues) -> Vec<SliderSegment<Quality>> {
    match segments_on(values) {
        true => vec![
            SliderSegment::labeled(Quality::Low, "Free"),
            SliderSegment::labeled(Quality::High, "Paid"),
        ],
        false => Vec::new(),
    }
}

fn continuous_segments(values: &DemoValues) -> Vec<SliderSegment> {
    match segments_on(values) {
        true => {
            let [min, _, _] = mark_values(values);
            vec![
                SliderSegment::labeled(min, "Comfortable"),
                SliderSegment::labeled(loud_from(values), "Loud"),
            ]
        }
        false => Vec::new(),
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

/// The readout, the other half of a controlled slider: `Start`/`End` bracket a drag,
/// `Change` carries each value. Printed too.
fn wrap_readout(values: &DemoValues, code: &str) -> String {
    let (declaration, signal, last) = match discrete(values) {
        true => (quality_code(), "quality():?", "last_quality()"),
        false => (String::new(), "volume()", "last()"),
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
                    prop("size", "Size").default(theme.slider.size.as_str()).doc("Track, thumb and font size."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Accent color. A theme color name or any CSS color."),
                    prop("value", "Option<V>").doc("The value. Pair it with `oninput`, or bind it with a path `name` inside a `Form`."),
                    prop("oninput", "EventHandler<SliderChangeEvent<V>>")
                        .doc("Fires per value while dragging. `Start` and `End` bracket a drag, `Change` carries each new value. A key press sends `Change`, then `End`."),
                    prop("min", "V")
                        .default("first option, or 0.0")
                        .doc("Lower bound, in the value's own type."),
                    prop("max", "V")
                        .default("last option, or 100.0")
                        .doc("Upper bound, in the value's own type. Off the `step` grid, the track ends at the last step below it, as on a native range input: 0 to 100 by 30 ends at 90."),
                    prop("step", "V::Step")
                        .default("one option, or 1.0")
                        .doc("How far one step goes from `min`. A count of options on a discrete scale, a value on a continuous one, so 0 to 1 has two stops unless you pass a smaller one. Also sets how many decimals a value keeps; `0.0` is continuous and keeps a thousandth of the range."),
                    prop("format", "Callback<V, String>")
                        .default("bare value, or SliderValue::label")
                        .doc("Text of the bubble and the thumb's `aria-valuetext`. On a discrete scale it also names the marks, so this is where a translation goes."),
                    prop("marks", "Vec<SliderMark<V>>")
                        .default("one per option, discretely")
                        .doc("Ticks on the track. A labeled one gets a caption below it. Replaces the marks a discrete scale draws itself. Past about six options the derived captions touch on a phone, so pass your own `marks`, or a `step` that skips options."),
                    prop("segments", "Vec<SliderSegment<V>>")
                        .default("[]")
                        .doc("Splits a line track into stretches with 2px gaps, each from its `start` to the next one's, as `Video`'s chapters. A labeled one is named after the value in the bubble and `aria-valuetext` (`slider.segment`, \"{value}, {segment}\"). Sorted for you; a start outside the track or a repeat is dropped, and an unlabeled stretch fills from `min` to the first start. A mark where two stretches meet keeps its caption but draws no dot: the gap is the tick."),
                    prop("aria_label", "String")
                        .doc("Names the thumb when the field has no `label`. Put in `attributes`, it would land on the wrapper instead."),
                    prop("name", "FieldName<V>")
                        .doc("Posts the value in a hidden input of that name. A discrete slider, whose type lists `options()`, posts the option's index, not the option. A path such as `Settings::FIELDS.volume()` also binds the value to the surrounding `Form`'s value when there is no `oninput`."),
                    prop("validate", "Validators<V>")
                        .doc("Rules over the value, shown once the slider loses focus or its form is submitted."),
                    prop("label", "Caption")
                        .doc("The caption above the track, and the thumb's name."),
                    prop("description", "Caption")
                        .doc("Between the label and the track. What the value means."),
                    prop("helper", "Caption")
                        .doc("Under the track, below the mark captions."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Adds an asterisk to the label. No `aria-required`: ARIA does not allow it on a slider, which always holds a value."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Takes the thumb out of the tab order and dims the slider."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable. `disabled` drops the slider from the tab order and the post instead."),
                    prop("track", "SliderTrack")
                        .default("Line")
                        .doc("How the track is drawn. `SliderTrack::Bars(heights)` draws a row of rounded bars, each as tall as its fraction (0 to 1) of a taller track, like `Audio`'s waveform; the bars up to the value fill in `color`."),
                ])
                .parts("SliderPart", vec![
                    (SliderPart::Label, "The label above the control."),
                    (SliderPart::Required, "The required asterisk, in the label."),
                    (SliderPart::Description, "The caption between the label and the control."),
                    (SliderPart::Control, "The slider under the label: the track and the room around it."),
                    (SliderPart::Track, "The rail the thumbs run along."),
                    (SliderPart::Bar, "The filled stretch of the track."),
                    (SliderPart::Bars, "The row of bars of a `SliderTrack::Bars` track."),
                    (SliderPart::Segments, "The row of segments a `segments` track draws."),
                    (SliderPart::Segment, "One stretch of a segmented track."),
                    (SliderPart::SegmentFill, "The filled part of a segment."),
                    (SliderPart::Mark, "One tick on the track."),
                    (SliderPart::MarkLabel, "A tick's caption."),
                    (SliderPart::Thumb, "The handle; a range has two."),
                    (SliderPart::Helper, "The caption under the control."),
                    (SliderPart::Status, "The validation message."),
                ]),
                props("SliderSegment", vec![
                    prop("start", "V").doc("Where the stretch starts; it ends at the next one's start, or `max`."),
                    prop("label", "Option<String>").doc("Names the values inside it. `SliderSegment::labeled(start, label)`."),
                ]),
                props("SliderMark", vec![
                    prop("value", "V").doc("Where the tick sits on the track."),
                    prop("label", "String").doc("Caption below the tick. Leave it out for a bare tick."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Left", "Right", "Up", "Down"], "Move `theme.slider.step` steps, one by default. With `step: 0.0` a step is 1% of the range. Right to left, ArrowLeft raises the value instead.")
                .key(["Shift+Arrow", "PageUp", "PageDown"], "Move `theme.slider.big_step` steps, ten by default.")
                .key(["Home", "End"], "Jump to the ends.")
                .handles([
                    "`format` replaces `SliderValue::label` in the bubble, the captions and `aria-valuetext`, and runs during render, so it can read the locale from context.",
                    "On a discrete slider the mark captions are hidden from screen readers, since the thumb already names each value. On a continuous one they stay, since they can say more than the number.",
                    "A `role=\"slider\"` takes no `aria-required`, so a `required` slider says the localization's `slider.required` word in its name instead, such as \"Volume required\". The asterisk stays hidden from screen readers.",
                ])
                .must([
                    "Without a `label`, set the `aria_label` prop. Put in `attributes`, it would name the wrapper instead of the thumb.",
                    "Pass `format` when a bare number does not say the unit, and to translate a discrete slider.",
                ])
                .example("A volume slider, `Slider { label: \"Volume\" }` with a `format` that adds \" %\": the arrows move one step, PageUp ten, Home and End jump to the ends, and a screen reader reads the formatted value."),
            lead: rsx! {
                Text {
                    "A value you drag along a track. It slides over any "
                    Code { source: "SliderValue" }
                    ". An "
                    Code { source: "f64" }
                    " gives a continuous range. An ordered enum that derives it makes the "
                    "slider discrete, and the range, steps, marks and captions come from its "
                    "options."
                }
            },
            // snippet: let mut volume = use_signal(|| 40.0);
            // snippet: let mut quality = use_signal(|| Quality::Medium);
            // snippet: let mut last = use_signal(|| SliderChangeEvent::Change(40.0));
            // snippet: let mut last_quality = use_signal(|| SliderChangeEvent::Change(Quality::Medium));
            Demo {
                component: "Slider",
                children_text: "",
                controls: vec![
                    // The value's type is the mode: an ordered enum is discrete, `f64` continuous.
                    // Each brings its own props, so the set below swaps with it.
                    Control::toggle("mode", ["discrete", "continuous"])
                        .labels(["Discrete", "Continuous"])
                        .code(mode_code),
                    Control::sizes("size").default(theme.slider.size.as_str()),
                    Control::color("color"),
                    Control::slider("min", Quality::ALL)
                        .options_from(min_options)
                        .code(bound_code)
                        .hidden_when(continuous),
                    Control::slider("max", Quality::ALL)
                        .options_from(max_options)
                        .default("ultra")
                        .code(bound_code)
                        .hidden_when(continuous),
                    Control::slider("step", ["1", "2", "3"])
                        .code(stride_code)
                        .hidden_when(continuous),
                    Control::slider("min_value", MIN_VALUES)
                        .options_from(min_value_options)
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("max_value", MAX_VALUES)
                        .options_from(max_value_options)
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("step_value", ["auto", "0.1", "5.0", "10.0", "25.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("Above the free tier.".into())"#
                                    .to_string(),
                            ],
                            "error" => vec![r#"status: "Pick a lower setting.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("bars").code(bars_code),
                    Control::switch("marks").code(marks_code),
                    Control::switch("segments").default("true").code(segments_code),
                    Control::switch("format").code(format_code),
                    Control::switch("label").default("true").code(|_, values| {
                        match (values.str("label").as_str(), discrete(values)) {
                            ("true", true) => vec![r#"label: "Quality""#.to_string()],
                            ("true", false) => vec![r#"label: "Volume""#.to_string()],
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
                                    color: values.str("color"),
                                    min: quality_of(&values, "min"),
                                    max: quality_of(&values, "max"),
                                    step: values.str("step").parse::<usize>().unwrap_or(1),
                                    marks: discrete_marks(&values),
                                    segments: discrete_segments(&values),
                                    track: track(&values),
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
                                    color: values.str("color"),
                                    min: number(&values, "min_value"),
                                    max: number(&values, "max_value"),
                                    step: number(&values, "step_value"),
                                    marks: continuous_marks(&values),
                                    segments: continuous_segments(&values),
                                    track: track(&values),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 1185: each bound offers only the options on its side of the other.
    #[test]
    fn the_bounds_never_cross() {
        let values = DemoValues::defaults(&[
            Control::slider("min", Quality::ALL).default("medium"),
            Control::slider("max", Quality::ALL).default("high"),
            Control::slider("min_value", MIN_VALUES).default("10.0"),
            Control::slider("max_value", MAX_VALUES).default("50.0"),
        ]);
        assert_eq!(min_options(&values), ["low", "medium"]);
        assert_eq!(max_options(&values), ["high", "ultra"]);
        assert_eq!(min_value_options(&values), ["auto", "0.5", "10.0"]);
        assert_eq!(max_value_options(&values), ["auto", "50.0", "200.0"]);
    }
}
