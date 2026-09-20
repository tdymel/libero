use crate::theme::Size;

/// Theme defaults for `TagsField`, set on [`Theme`](crate::theme::Theme).
/// The frame lives on `FieldDefaults`, the list on `ComboboxDefaults`; chips take one step down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagsFieldDefaults {
    pub size: Size,
    pub radius: Size,
}

impl TagsFieldDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
    };
}
