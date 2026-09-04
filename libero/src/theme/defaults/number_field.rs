use crate::theme::Size;

/// What `NumberField` does not share with every other field. The frame's
/// numbers live on `FieldDefaults`, so a `NumberField` beside a `TextField`
/// lines up with it by construction.
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
