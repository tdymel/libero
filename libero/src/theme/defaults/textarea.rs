use crate::theme::Size;

/// Theme defaults for `Textarea`, set on [`Theme`](crate::theme::Theme).
/// The frame's numbers live on `FieldDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextareaDefaults {
    pub size: Size,
    pub radius: Size,
}

impl TextareaDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
    };
}
