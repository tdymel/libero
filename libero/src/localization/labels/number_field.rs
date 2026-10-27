/// `NumberField`'s steppers and touch spinner, when its props are unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberFieldLabels {
    pub decrease: &'static str,
    pub increase: &'static str,
    /// Closes the touch spinner, its steps already sent.
    pub done: &'static str,
    /// Names the spinner's keyboard button, which switches to typing.
    pub keyboard: &'static str,
    /// The error for typed text that is no number, such as `1.5` where `.` groups.
    pub invalid_number: &'static str,
}

impl NumberFieldLabels {
    pub const ENGLISH: Self = Self {
        decrease: "Decrease",
        increase: "Increase",
        done: "Done",
        keyboard: "Type a number",
        invalid_number: "Enter a number.",
    };

    pub const GERMAN: Self = Self {
        decrease: "Verringern",
        increase: "Erhöhen",
        done: "Fertig",
        keyboard: "Zahl eingeben",
        invalid_number: "Geben Sie eine Zahl ein.",
    };
}
