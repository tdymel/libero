use super::slider_demo::{
    Quality, bound_code, continuous, continuous_marks, discrete, discrete_marks, is_on, marks_code,
    max_options, min_options, number, number_code, quality_code, quality_of, within,
};
use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, Wrap, a11y, field_controls, field_props, indent,
    prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::SliderPart;
use libero::{
    components::{Code, Flex, RangeSlider, SliderChangeEvent, Text},
    sx::sx,
    use_theme,
};

struct PriceCopy;

impl FieldCopy for PriceCopy {
    const LABEL: &'static str = "Price";
    const DESCRIPTION: &'static str = "What you are willing to pay.";
    const HELPER: &'static str = "Both ends included.";
    const WARNING: &'static str = "A wide range costs more.";
    const ERROR: &'static str = "Pick a narrower range.";
}

const MIN_VALUES: [&str; 4] = ["auto", "0.0", "10.0", "50.0"];
const MAX_VALUES: [&str; 4] = ["auto", "50.0", "100.0", "200.0"];

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

/// The continuous twins of `min_options` and `max_options`; `auto` is 0 and 100.
fn min_value_options(values: &DemoValues) -> Vec<String> {
    let max = number(values, "max_value").unwrap_or(100.0);
    within(&MIN_VALUES, |min| min < max)
}

fn max_value_options(values: &DemoValues) -> Vec<String> {
    let min = number(values, "min_value").unwrap_or(0.0);
    within(&MAX_VALUES, |max| max > min)
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

/// The readout, the other half of a controlled range, shows the thumbs never cross. Printed too.
fn wrap_readout(values: &DemoValues, code: &str) -> String {
    let (declaration, signal, last) = match discrete(values) {
        true => (quality_code(), "quality():?", "last_quality()"),
        false => (String::new(), "price():?", "last()"),
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
                    prop("size", "Size").default(theme.slider.size.as_str()).doc("Track, thumb and font size."),
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
                        .doc("Upper bound of the track, in the value's own type. Off the `step` grid, the track ends at the last step below it, as on a native range input: 0 to 100 by 30 ends at 90."),
                    prop("step", "V::Step")
                        .default("one option, or 1.0")
                        .doc("How far one step goes from `min`. A count of options on a discrete scale, a value on a continuous one, so 0 to 1 has two stops unless you pass a smaller one. Also sets how many decimals a value keeps; `0.0` is continuous and keeps a thousandth of the range."),
                    prop("min_range", "V::Step")
                        .default("0")
                        .doc("The smallest gap the thumbs keep, in the unit of `step`. At 0 they may meet, and they never cross."),
                    prop("format", "Callback<V, String>")
                        .default("bare value, or SliderValue::label")
                        .doc("Text of the bubbles and each thumb's `aria-valuetext`. On a discrete scale it also names the marks, so this is where a translation goes."),
                    prop("marks", "Vec<SliderMark<V>>")
                        .default("one per option, discretely")
                        .doc("Ticks on the track. A labeled one gets a caption below it. Replaces the marks a discrete scale draws itself. Past about six options the derived captions touch on a phone, so pass your own `marks`, or a `step` that skips options."),
                    prop("aria_label", "String")
                        .doc("Names the pair when the field has no `label`: the group, and each thumb before its own word. Put in `attributes`, it would land on the wrapper instead."),
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
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Adds an asterisk to the label. No `aria-required`: ARIA does not allow it on a slider, which always holds a value."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Takes the thumbs out of the tab order and dims the slider."),
                    readonly_prop("slider"),
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
                .key(["Left", "Right", "Up", "Down"], "Move the focused thumb `theme.slider.step` steps, one by default. Right to left, ArrowLeft raises the value instead.")
                .key(["Shift+Arrow", "PageUp", "PageDown"], "Move the focused thumb `theme.slider.big_step` steps, ten by default.")
                .key(["Home", "End"], "Move the focused thumb to the end, stopping at the other thumb.")
                .handles([
                    "The two thumbs sit in a `role=\"group\"` named by the label, and each is its own `role=\"slider\"`, as in the ARIA multi-thumb slider pattern. A single `Slider` is one slider and needs no group.",
                    "Each thumb is named by the label plus its own word, such as \"Price Minimum\" and \"Price Maximum\", from the localization's `slider.minimum` and `slider.maximum`. Without a `label`, `aria_label` names the group and the thumbs the same way.",
                    "On a discrete range the mark captions are hidden from screen readers, since the thumbs already name each value.",
                    "A `role=\"slider\"` takes no `aria-required`, so a `required` range says the localization's `slider.required` word in each thumb's name instead, such as \"Price required Minimum\". The asterisk stays hidden from screen readers.",
                ])
                .must([
                    "Without a `label`, set the `aria_label` prop, or the thumbs say only \"Minimum\" and \"Maximum\". Put in `attributes`, it would name a wrapper with no role.",
                    "Set `aria_label_from` and `aria_label_to` when those words do not fit.",
                    "Set `format` when a bare number does not say the unit. To translate a discrete range, pass `format`, as on a `Slider`.",
                ])
                .example("A price range, `RangeSlider { label: \"Price\" }`: the thumbs read as \"Price Minimum\" and \"Price Maximum\". The arrows move the focused thumb one step, PageUp ten, and Home or End to the end, stopping at the other thumb."),
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
                controls: [vec![
                    Control::toggle("mode", ["discrete", "continuous"])
                        .default("continuous")
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
                    Control::slider("min_range", ["0", "1", "2"])
                        .code(min_range_code)
                        .hidden_when(continuous),
                    Control::slider("min_value", MIN_VALUES)
                        .options_from(min_value_options)
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("max_value", MAX_VALUES)
                        .options_from(max_value_options)
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("step_value", ["auto", "0.5", "5.0", "10.0", "25.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                    Control::slider("min_range_value", ["0.0", "10.0", "25.0"])
                        .code(number_code)
                        .hidden_when(discrete),
                ], field_controls::<PriceCopy>(), vec![
                    Control::switch("marks").code(marks_code),
                    Control::switch("format").code(format_code),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<PriceCopy>(&values);
                    rsx! {
                        Flex {
                            direction: "column",
                            gap: "sm",
                            sx: sx().width("100%"),
                            if discrete(&values) {
                                RangeSlider {
                                    aria_label: field.aria_label.map(String::from),
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
                                    label: field.label.clone(),
                                    description: field.description.clone(),
                                    helper: field.helper.clone(),
                                    status: field.status.clone(),
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
                                    aria_label: field.aria_label.map(String::from),
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
                                    label: field.label,
                                    description: field.description,
                                    helper: field.helper,
                                    status: field.status,
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
