//! Rosé Pine - Rosé Pine and Rosé Pine Dawn.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Rosé Pine Dawn.
pub static ROSE_PINE_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFAF4ED),
    ink: HexColor::new(0x575279),
    neutral: HexColor::new(0x575279),
    muted: HexColor::new(0x6E6A86),
    primary: HexColor::new(0x286983),
    secondary: HexColor::new(0x9076C4),
    info: HexColor::new(0x286983),
    success: HexColor::new(0x569486),
    error: HexColor::new(0xB4637A),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#fffaf3",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Rosé Pine.
pub static ROSE_PINE_DARK: Theme = Theme {
    surface: HexColor::new(0x191724),
    ink: HexColor::new(0xE0DEF4),
    neutral: HexColor::new(0xE0DEF4),
    muted: HexColor::new(0x908CAA),
    primary: HexColor::new(0x31748F),
    secondary: HexColor::new(0xC4A7E7),
    info: HexColor::new(0xC4A7E7),
    success: HexColor::new(0x9CCFD8),
    error: HexColor::new(0xEB6F92),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#1f1d2e",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Rosé Pine: [`ROSE_PINE_LIGHT`] paired with [`ROSE_PINE_DARK`].
    pub const ROSE_PINE: ThemeSet = ThemeSet::pair("Rosé Pine", &ROSE_PINE_LIGHT, &ROSE_PINE_DARK);
}
