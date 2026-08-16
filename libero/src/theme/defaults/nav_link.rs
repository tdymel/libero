use crate::theme::Color;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavLinkDefaults {
    pub color: Color,
}

impl NavLinkDefaults {
    pub const fn new(color: Color) -> Self {
        Self { color }
    }
}
