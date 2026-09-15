use crate::theme::{Color, Variant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorSchemeButtonDefaults {
    /// `Outlined`, so it reads as a control with a box of its own rather
    /// than a bare glyph in a header - Mantine's `default` action icon.
    pub variant: Variant,
    /// `Muted` keeps the border quiet: the accent belongs to the page, not to
    /// the chrome around it.
    pub color: Color,
}

impl ColorSchemeButtonDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Muted,
    };
}
