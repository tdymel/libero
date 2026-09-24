/// `PasswordField`'s reveal button, when its props are unset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasswordFieldLabels {
    /// Names the button in both states; `aria-pressed` carries the state.
    pub show: &'static str,
}

impl PasswordFieldLabels {
    pub const ENGLISH: Self = Self {
        show: "Show password",
    };

    pub const GERMAN: Self = Self {
        show: "Passwort anzeigen",
    };
}
