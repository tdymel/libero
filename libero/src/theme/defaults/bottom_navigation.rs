use crate::theme::Color;

/// Theme defaults for `BottomNavigation`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BottomNavigationDefaults {
    pub color: Color,
}

impl BottomNavigationDefaults {
    pub const DEFAULT: Self = Self {
        color: Color::Primary,
    };
}
