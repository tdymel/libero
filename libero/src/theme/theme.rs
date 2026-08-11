use super::{HexColor, Sizes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub spacing: Sizes<u8>,
    pub primary: HexColor,
    pub secondary: HexColor,
    pub black: HexColor,
    pub white: HexColor,
}

impl Theme {
    pub const DEFAULT: Theme = Theme::new(
        Sizes::new(4, 8, 12, 16, 20),
        HexColor::new(0x228BE6),
        HexColor::new(0xE03131),
        HexColor::new(0x000000),
        HexColor::new(0xFFFFFF),
    );

    pub const fn new(
        spacing: Sizes<u8>,
        primary: HexColor,
        secondary: HexColor,
        black: HexColor,
        white: HexColor,
    ) -> Self {
        Self {
            spacing,
            primary,
            secondary,
            black,
            white,
        }
    }
}
