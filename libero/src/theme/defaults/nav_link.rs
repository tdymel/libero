use crate::theme::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavLinkDefaults {
    pub color: Color,
}

impl NavLinkDefaults {
    pub const DEFAULT: Self = Self {
        color: Color::Primary,
    };
}
