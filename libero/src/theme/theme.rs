use super::{HexColor, Sizes, StackAxisDefaults, StackDefaults};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub spacing: Sizes<u8>,
    pub stack: StackDefaults,
    pub primary: HexColor,
    pub secondary: HexColor,
    pub error: HexColor,
    pub warning: HexColor,
    pub info: HexColor,
    pub success: HexColor,
    pub grey: HexColor,
    pub black: HexColor,
    pub white: HexColor,
}

impl Theme {
    pub const DEFAULT: Theme = Theme::new(
        Sizes::new(4, 8, 12, 16, 20),
        StackDefaults::new(
            StackAxisDefaults::new("stretch", "flex-start", super::Size::Md, false),
            StackAxisDefaults::new("center", "flex-start", super::Size::Md, true),
        ),
        HexColor::new(0x228BE6),
        HexColor::new(0xE03131),
        HexColor::new(0xE03131),
        HexColor::new(0xF08C00),
        HexColor::new(0x228BE6),
        HexColor::new(0x2F9E44),
        HexColor::new(0x868E96),
        HexColor::new(0x000000),
        HexColor::new(0xFFFFFF),
    );

    pub const fn new(
        spacing: Sizes<u8>,
        stack: StackDefaults,
        primary: HexColor,
        secondary: HexColor,
        error: HexColor,
        warning: HexColor,
        info: HexColor,
        success: HexColor,
        grey: HexColor,
        black: HexColor,
        white: HexColor,
    ) -> Self {
        Self {
            spacing,
            stack,
            primary,
            secondary,
            error,
            warning,
            info,
            success,
            grey,
            black,
            white,
        }
    }
}
