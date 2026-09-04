use crate::theme::Size;

/// What `Autocomplete` does not share with every other field. The frame's
/// numbers live on `FieldDefaults` and the dropdown's on `ComboboxDefaults`,
/// so a suggestion field lines up with a `TextField` above it and with a
/// `Select`'s list below it by construction.
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
