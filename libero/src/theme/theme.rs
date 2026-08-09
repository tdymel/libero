use super::{HexColor, Sizes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Theme {
    pub spacing: Sizes<u8>,
    pub primary: HexColor,
    pub secondary: HexColor,
}

impl Theme {
    pub const fn new(spacing: Sizes<u8>, primary: HexColor, secondary: HexColor) -> Self {
        Self {
            spacing,
            primary,
            secondary,
        }
    }
}
