use crate::theme::Size;

/// What `NativeSelect` does not share with every other field. The frame's numbers -
/// font size, height, padding, radius scale - live on `FieldDefaults`, so a
/// `NativeSelect` and a `TextField` line up in one form by construction rather than
/// by two tables agreeing.
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
