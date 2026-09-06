/// What `PasswordField` adds to the `TextField` it renders. The frame's
/// numbers are the text field's.
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
