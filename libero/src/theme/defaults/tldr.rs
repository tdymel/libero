use crate::theme::{Color, Size, Variant};

/// Theme defaults for `Tldr`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TldrDefaults {
    /// `Outlined`.
    pub variant: Variant,
    /// `Neutral`.
    pub color: Color,
    /// The trigger's size step, the icon-only one too.
    pub size: Size,
    /// `None`: the trigger's own, `xl` on the chip and `sm` icon-only.
    pub radius: Option<Size>,
}

impl TldrDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Outlined,
        color: Color::Neutral,
        size: Size::Md,
        radius: None,
    };
}
