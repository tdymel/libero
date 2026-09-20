/// Theme defaults for `PasswordField`, set on [`Theme`](crate::theme::Theme).
/// The frame's numbers are the `TextField`'s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasswordFieldDefaults {
    /// Offers the reveal button when a call site does not say.
    pub reveal_button: bool,
}

impl PasswordFieldDefaults {
    pub const DEFAULT: Self = Self {
        reveal_button: true,
    };
}
