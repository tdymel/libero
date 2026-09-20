use crate::theme::Color;

/// Theme defaults for `NavLink`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavLinkDefaults {
    pub color: Color,
}

impl NavLinkDefaults {
    pub const DEFAULT: Self = Self {
        color: Color::Primary,
    };
}
