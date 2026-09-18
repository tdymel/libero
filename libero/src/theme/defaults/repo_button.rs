use crate::theme::{Color, Variant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepoButtonDefaults {
    /// `Outlined`, the header chrome `ColorSchemeButton` wears.
    pub variant: Variant,
    /// `Muted`: the accent belongs to the page, not to the chrome around it.
    pub color: Color,
}

impl RepoButtonDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Muted,
    };
}
