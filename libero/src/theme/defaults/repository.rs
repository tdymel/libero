use crate::theme::{Color, Variant};

/// Theme defaults for `Repository`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepositoryDefaults {
    /// `Outlined`, the header chrome `ThemeSwitcher` wears.
    pub variant: Variant,
    /// `Muted`: the accent belongs to the page, not to the chrome around it.
    pub color: Color,
}

impl RepositoryDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Muted,
    };
}
