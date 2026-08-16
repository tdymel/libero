use crate::theme::Color;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkDefaults {
    pub color: Color,
}

impl MarkDefaults {
    pub const fn new(color: Color) -> Self {
        Self { color }
    }
}
