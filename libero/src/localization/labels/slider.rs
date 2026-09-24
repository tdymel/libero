/// `RangeSlider`'s thumbs, when its props are unset. Each follows the field's
/// label: "Price Minimum".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliderLabels {
    pub minimum: &'static str,
    pub maximum: &'static str,
    /// Said after a required slider's label; a thumb takes no `aria-required`.
    pub required: &'static str,
}

impl SliderLabels {
    pub const ENGLISH: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
        required: "required",
    };

    pub const GERMAN: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
        required: "erforderlich",
    };
}
