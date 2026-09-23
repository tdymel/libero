use crate::theme::{Color, Variant};

/// Theme defaults for `Tldr`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TldrDefaults {
    /// `Outlined`.
    pub variant: Variant,
    /// `Neutral`.
    pub color: Color,
}

impl TldrDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Neutral,
    };
}
