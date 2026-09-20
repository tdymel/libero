use crate::theme::Size;

/// Theme defaults for `Select` and `MultiSelect`, set on [`Theme`](crate::theme::Theme).
/// The frame's numbers live on `FieldDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectDefaults {
    pub size: Size,
    pub radius: Size,
}

impl SelectDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
    };
}
