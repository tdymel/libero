/// `RangeSlider`'s thumbs, when its props are unset. Each follows the field's
/// label: "Price Minimum".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliderLabels {
    pub minimum: &'static str,
    pub maximum: &'static str,
    /// Said after a required slider's label; a thumb takes no `aria-required`.
    pub required: &'static str,
    /// A value inside a labelled segment: `{value}`, then the `{segment}`'s label.
    pub segment: &'static str,
}

impl SliderLabels {
    pub const ENGLISH: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
        required: "required",
        segment: "{value}, {segment}",
    };

    pub const GERMAN: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
        required: "erforderlich",
        segment: "{value}, {segment}",
    };
}
