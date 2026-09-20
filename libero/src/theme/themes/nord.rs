//! Nord: the Kopuz pack's dark, and the bright ambiance Nord's docs describe
//! (<https://www.nordtheme.com/docs/colors-and-palettes>) as Helix ships it.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Nord, bright ambiance.
pub static NORD_LIGHT: Theme = Theme {
    surface: HexColor::new(0xECEFF4),
    ink: HexColor::new(0x2E3440),
    neutral: HexColor::new(0x2E3440),
    muted: HexColor::new(0x4C566A),
    primary: HexColor::new(0x88C0D0),
    secondary: HexColor::new(0xB48EAD),
    info: HexColor::new(0x8FBCBB),
    success: HexColor::new(0xA3BE8C),
    error: HexColor::new(0xBF616A),
    paper: PaperDefaults {
        background: "#d8dee9",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Nord.
pub static NORD_DARK: Theme = Theme {
    surface: HexColor::new(0x2E3440),
    ink: HexColor::new(0xD8DEE9),
    neutral: HexColor::new(0xD8DEE9),
    muted: HexColor::new(0x81A1C1),
    primary: HexColor::new(0x88C0D0),
    secondary: HexColor::new(0xB48EAD),
    info: HexColor::new(0x8FBCBB),
    success: HexColor::new(0xA3BE8C),
    error: HexColor::new(0xBF616A),
    paper: PaperDefaults {
        background: "#3b4252",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Nord: [`NORD_LIGHT`] paired with [`NORD_DARK`].
    pub const NORD: ThemeSet = ThemeSet::pair("Nord", &NORD_LIGHT, &NORD_DARK);
}
