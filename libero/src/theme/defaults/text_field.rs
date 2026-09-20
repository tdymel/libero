use crate::theme::Size;

/// Theme defaults for `TextField`, set on [`Theme`](crate::theme::Theme).
/// The frame's numbers live on `FieldDefaults`.
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
