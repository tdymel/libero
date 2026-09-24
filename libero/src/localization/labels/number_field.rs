/// `NumberField`'s steppers, when its props are unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberFieldLabels {
    pub decrease: &'static str,
    pub increase: &'static str,
}

impl NumberFieldLabels {
    pub const ENGLISH: Self = Self {
        decrease: "Decrease",
        increase: "Increase",
    };

    pub const GERMAN: Self = Self {
        decrease: "Verringern",
        increase: "Erhöhen",
    };
}
