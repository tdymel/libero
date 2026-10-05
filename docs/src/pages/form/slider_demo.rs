//! What the `Slider` and `RangeSlider` demos share: the discrete `Quality` scale and the
//! helpers behind their controls.
use crate::components::{Control, DemoFile, DemoValues};
use libero::components::{SliderMark, SliderValue};

const FILE: DemoFile = DemoFile(include_str!("slider_demo.rs"));

/// The demos' own discrete type - a range slides over the same values a single thumb does.
// demo-code: quality start
#[derive(Clone, Copy, Debug, PartialEq, SliderValue)]
pub(super) enum Quality {
    Low,
    Medium,
    High,
    #[slider(label = "Max")]
    Ultra,
}
// demo-code: quality end

impl Quality {
    pub(super) const ALL: [&'static str; 4] = ["low", "medium", "high", "ultra"];

    pub(super) fn parse(value: &str) -> Self {
        match value {
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            _ => Self::Ultra,
        }
    }
}

/// Printed above the rsx in discrete mode: without the derive there is no discrete slider.
pub(super) fn quality_code() -> String {
    FILE.section("quality").replacen("pub(super) ", "", 1) + "\n\n"
}

pub(super) fn discrete(values: &DemoValues) -> bool {
    values.str("mode") == "discrete"
}

pub(super) fn continuous(values: &DemoValues) -> bool {
    !discrete(values)
}

pub(super) fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

pub(super) fn number(values: &DemoValues, name: &str) -> Option<f64> {
    values.str(name).parse::<f64>().ok()
}

pub(super) fn quality_of(values: &DemoValues, name: &str) -> Quality {
    Quality::parse(&values.str(name))
}

/// A bound in the value's own type. Quiet at the option the slider would pick
/// anyway - the first for `min`, the last for `max`.
pub(super) fn bound_code(control: &Control, values: &DemoValues) -> Vec<String> {
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
pub(super) fn number_code(control: &Control, values: &DemoValues) -> Vec<String> {
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

/// `min` offers the qualities below `max`, and `max` those above `min`: the bounds never cross.
pub(super) fn min_options(values: &DemoValues) -> Vec<String> {
    let max = Quality::parse(&values.str("max")) as usize;
    Quality::ALL[..max]
        .iter()
        .map(|quality| quality.to_string())
        .collect()
}

pub(super) fn max_options(values: &DemoValues) -> Vec<String> {
    let min = Quality::parse(&values.str("min")) as usize;
    Quality::ALL[min + 1..]
        .iter()
        .map(|quality| quality.to_string())
        .collect()
}

/// `auto`, and the numbers `keep` accepts.
pub(super) fn within(options: &[&str], keep: impl Fn(f64) -> bool) -> Vec<String> {
    options
        .iter()
        .filter(|option| option.parse().map_or(true, &keep))
        .map(|option| option.to_string())
        .collect()
}

/// Ticks at both ends and the midpoint of the *current* range - a mark at 50
/// means nothing on a 0-to-10 slider.
pub(super) fn mark_values(values: &DemoValues) -> [f64; 3] {
    let min = values.str("min_value").parse().unwrap_or(0.0);
    let max = values.str("max_value").parse().unwrap_or(100.0);
    [min, (min + max) / 2.0, max]
}

/// Discretely, `marks` *replaces* the one-per-option set the type derives;
/// continuously there is nothing to derive, so it adds them.
pub(super) fn marks_code(_control: &Control, values: &DemoValues) -> Vec<String> {
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

pub(super) fn discrete_marks(values: &DemoValues) -> Vec<SliderMark<Quality>> {
    match is_on(values, "marks") {
        true => vec![
            SliderMark::labeled(Quality::Low, "cheap"),
            SliderMark::labeled(Quality::Ultra, "pricey"),
        ],
        false => Vec::new(),
    }
}

pub(super) fn continuous_marks(values: &DemoValues) -> Vec<SliderMark> {
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
