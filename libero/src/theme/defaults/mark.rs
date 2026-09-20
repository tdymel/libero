use crate::theme::Color;

/// Theme defaults for `Mark`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkDefaults {
    pub color: Color,
}

impl MarkDefaults {
    pub const DEFAULT: Self = Self {
        color: Color::Warning,
    };
}
