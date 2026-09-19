use crate::theme::{Color, Variant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirectionToggleDefaults {
    /// `Outlined`, the header chrome `ThemeToggle` wears.
    pub variant: Variant,
    /// `Muted`: the accent belongs to the page, not to the chrome around it.
    pub color: Color,
}

impl DirectionToggleDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Muted,
    };
}
