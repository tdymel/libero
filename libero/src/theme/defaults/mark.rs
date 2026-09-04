use crate::theme::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkDefaults {
    pub color: Color,
}

impl MarkDefaults {
    pub const DEFAULT: Self = Self {
        color: Color::Warning,
    };
}
