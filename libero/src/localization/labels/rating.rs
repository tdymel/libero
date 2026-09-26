/// `Rating`'s spoken value, when its `format` is unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RatingLabels {
    /// `aria-valuetext`, and a display-only rating's name. `{value}` is the
    /// value, `{count}` the number of symbols: "3.5 of 5".
    pub value: &'static str,
    /// Said after a required rating's label; a slider takes no `aria-required`.
    pub required: &'static str,
}

impl RatingLabels {
    pub const ENGLISH: Self = Self {
        value: "{value} of {count}",
        required: "required",
    };

    pub const GERMAN: Self = Self {
        value: "{value} von {count}",
        required: "erforderlich",
    };
}
