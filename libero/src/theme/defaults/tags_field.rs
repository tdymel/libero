use crate::theme::Size;

/// What `TagsField` does not share with every other field. The frame's numbers
/// live on `FieldDefaults` and the dropdown's on `ComboboxDefaults`, so a tags
/// field lines up with a `TextField` above it and with a `Select`'s list below
/// it by construction. The chips take one step down this scale, which is the
/// component's own arithmetic rather than a theme key.
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
