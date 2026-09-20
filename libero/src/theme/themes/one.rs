//! One - One Dark Pro and One Light, Atom's palettes.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// One Light.
pub static ONE_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFAFAFA),
    ink: HexColor::new(0x383A42),
    neutral: HexColor::new(0x383A42),
    muted: HexColor::new(0x696C77),
    primary: HexColor::new(0x4078F2),
    secondary: HexColor::new(0xA626A4),
    info: HexColor::new(0x0184BC),
    success: HexColor::new(0x50A14F),
    error: HexColor::new(0xE45649),
    paper: PaperDefaults {
        background: "#f0f0f0",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// One Dark Pro.
pub static ONE_DARK: Theme = Theme {
    surface: HexColor::new(0x282C34),
    ink: HexColor::new(0xABB2BF),
    neutral: HexColor::new(0xABB2BF),
    muted: HexColor::new(0x7F848E),
    primary: HexColor::new(0x61AFEF),
    secondary: HexColor::new(0xC678DD),
    info: HexColor::new(0x61AFEF),
    success: HexColor::new(0x98C379),
    error: HexColor::new(0xE06C75),
    paper: PaperDefaults {
        background: "#21252b",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// One: [`ONE_LIGHT`] paired with [`ONE_DARK`].
    pub const ONE: ThemeSet = ThemeSet::pair("One", &ONE_LIGHT, &ONE_DARK);
}
