use crate::theme::Size;

/// Theme defaults for `NativeSelect`, set on [`Theme`](crate::theme::Theme).
/// The frame's numbers live on `FieldDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeSelectDefaults {
    pub size: Size,
    pub radius: Size,
}

impl NativeSelectDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
    };
}
