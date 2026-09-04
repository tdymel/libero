use crate::theme::Size;

/// What `TextField` does not share with every other field. The frame's
/// numbers - font size, height, padding, radius scale - live on
/// `FieldDefaults`, so a `TextField` and a `NativeSelect` line up in one form by
/// construction rather than by two tables agreeing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFieldDefaults {
    pub size: Size,
    pub radius: Size,
}

impl TextFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
    };
}
