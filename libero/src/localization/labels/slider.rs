/// `RangeSlider`'s thumbs, when its props are unset. Each follows the field's
/// label: "Price Minimum".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliderLabels {
    pub minimum: &'static str,
    pub maximum: &'static str,
}

impl SliderLabels {
    pub const ENGLISH: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
    };

    pub const GERMAN: Self = Self {
        minimum: "Minimum",
        maximum: "Maximum",
    };
}
