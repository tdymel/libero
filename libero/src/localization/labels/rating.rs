/// `Rating`'s spoken value, when its `format` is unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RatingLabels {
    /// `aria-valuetext`, and a display-only rating's name. `{value}` is the
    /// value, `{count}` the number of symbols: "3.5 of 5".
    pub value: &'static str,
}

impl RatingLabels {
    pub const ENGLISH: Self = Self {
        value: "{value} of {count}",
    };

    pub const GERMAN: Self = Self {
        value: "{value} von {count}",
    };
}
