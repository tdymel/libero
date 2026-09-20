use crate::theme::Size;

/// Theme defaults for `NumberField`, set on [`Theme`](crate::theme::Theme).
/// The frame's numbers live on `FieldDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberFieldDefaults {
    pub size: Size,
    pub radius: Size,
}

impl NumberFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
    };
}
