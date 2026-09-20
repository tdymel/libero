use crate::theme::Size;

/// Theme defaults for `Autocomplete`, set on [`Theme`](crate::theme::Theme).
///
/// Frame and dropdown come from `FieldDefaults` and `ComboboxDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutocompleteDefaults {
    pub size: Size,
    pub radius: Size,
}

impl AutocompleteDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
    };
}
