use crate::theme::{Color, Variant};

/// Theme defaults for `ThemeToggle`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeToggleDefaults {
    /// `Outlined`, so it reads as a control rather than a bare glyph.
    pub variant: Variant,
    /// `Muted`: the accent belongs to the page, not to the chrome around it.
    pub color: Color,
}

impl ThemeToggleDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Muted,
    };
}
