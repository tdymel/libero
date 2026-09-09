//! Everforest - Everforest's medium-contrast pair.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Everforest Light.
pub static EVERFOREST_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFDF6E3),
    ink: HexColor::new(0x5C6A72),
    neutral: HexColor::new(0x5C6A72),
    muted: HexColor::new(0x829181),
    primary: HexColor::new(0x3A94C5),
    secondary: HexColor::new(0xDF69BA),
    info: HexColor::new(0x35A77C),
    success: HexColor::new(0x8DA101),
    error: HexColor::new(0xF85552),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#f8f0dc",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Everforest.
pub static EVERFOREST_DARK: Theme = Theme {
    surface: HexColor::new(0x272E33),
    ink: HexColor::new(0xD3C6AA),
    neutral: HexColor::new(0xD3C6AA),
    muted: HexColor::new(0x859289),
    primary: HexColor::new(0x7FBBB3),
    secondary: HexColor::new(0xD699B6),
    info: HexColor::new(0x7FBBB3),
    success: HexColor::new(0xA7C080),
    error: HexColor::new(0xE67E80),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#2d353b",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Everforest: [`EVERFOREST_LIGHT`] paired with [`EVERFOREST_DARK`].
    pub const EVERFOREST: ThemeSet =
        ThemeSet::pair("Everforest", &EVERFOREST_LIGHT, &EVERFOREST_DARK);
}
