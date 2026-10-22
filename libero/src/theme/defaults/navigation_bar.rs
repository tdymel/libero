use crate::theme::Color;

/// Theme defaults for `NavigationBar`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationBarDefaults {
    pub color: Color,
}

impl NavigationBarDefaults {
    pub const DEFAULT: Self = Self {
        color: Color::Primary,
    };
}
