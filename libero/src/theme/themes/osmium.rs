//! Osmium - a derived light half, and IroncladDev's Osmium dark.
//!
//! The dark half matches `@webtui/theme-osmium`. Dark-only upstream, so the light half
//! is ours, by the rule in [`vague`](super::VAGUE_LIGHT):
//!
//! - `ink` is the dark `bg`, `#14131e`; page and card are `#c8d5f1` mixed 80%/60% to white;
//! - `muted` is `#949bb9` moved toward the ink to the same 6.70:1 from the page.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Osmium, light - derived, not upstream.
pub static OSMIUM_LIGHT: Theme = Theme {
    surface: HexColor::new(0xF4F7FC),
    ink: HexColor::new(0x14131E),
    neutral: HexColor::new(0x14131E),
    muted: HexColor::new(0x54566B),
    primary: HexColor::new(0xB0A8EB),
    secondary: HexColor::new(0xD9A1E8),
    info: HexColor::new(0x9ABFE8),
    success: HexColor::new(0xC9DE96),
    error: HexColor::new(0xE55376),
    paper: PaperDefaults {
        background: "#e9eef9",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Osmium.
pub static OSMIUM_DARK: Theme = Theme {
    surface: HexColor::new(0x14131E),
    ink: HexColor::new(0xC8D5F1),
    neutral: HexColor::new(0xC8D5F1),
    muted: HexColor::new(0x949BB9),
    primary: HexColor::new(0xB0A8EB),
    secondary: HexColor::new(0xD9A1E8),
    info: HexColor::new(0x9ABFE8),
    success: HexColor::new(0xC9DE96),
    error: HexColor::new(0xE55376),
    paper: PaperDefaults {
        background: "#1f1d2d",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Osmium: [`OSMIUM_LIGHT`] (derived) paired with [`OSMIUM_DARK`].
    pub const OSMIUM: ThemeSet = ThemeSet::pair("Osmium", &OSMIUM_LIGHT, &OSMIUM_DARK);
}
