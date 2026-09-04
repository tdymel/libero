use crate::theme::Size;

/// What `Select` and `MultiSelect` do not share with every other field. The
/// frame's numbers live on `FieldDefaults`, so either lines up with a
/// `TextField` in one form by construction.
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
