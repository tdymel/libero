//! Vague - a derived light half, and Vague dark (`vague-theme/vague.nvim`).
//!
//! The dark half is the Kopuz pack's. Vague is dark-only upstream - its light
//! theme is an open request (vague.nvim issue 90) - and the one community
//! light port, `Rnedlose/vague-light.nvim`, is gone, so **the light half is
//! ours**, derived from the dark half by the rule Osmium and kettek16 share
//! (the module doc of [`super`] states it once):
//!
//! - the five accents are the dark half's own - the stylesheet derives every
//!   role's text and fill steps against the page, so a hue needs no retuning;
//! - `ink` is the dark `bg`, `#141415`;
//! - the page is the dark `text` (`#cdcdcd`) mixed 80% toward white, the card
//!   the same mixed 60%, so the card sits one step off the page;
//! - `muted` is the dark `text-muted` (`#606079`) moved toward white until it
//!   is as far from the light page (2.99:1) as it is from the dark one.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Vague, light - derived, not upstream.
pub static VAGUE_LIGHT: Theme = Theme {
    surface: HexColor::new(0xF5F5F5),
    ink: HexColor::new(0x141415),
    neutral: HexColor::new(0x141415),
    muted: HexColor::new(0x8D8D9F),
    primary: HexColor::new(0x6E94B2),
    secondary: HexColor::new(0xBB9DBD),
    info: HexColor::new(0x7E98E8),
    success: HexColor::new(0x7FA563),
    error: HexColor::new(0xD8647E),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#ebebeb",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Vague.
pub static VAGUE_DARK: Theme = Theme {
    surface: HexColor::new(0x141415),
    ink: HexColor::new(0xCDCDCD),
    neutral: HexColor::new(0xCDCDCD),
    muted: HexColor::new(0x606079),
    primary: HexColor::new(0x6E94B2),
    secondary: HexColor::new(0xBB9DBD),
    info: HexColor::new(0x7E98E8),
    success: HexColor::new(0x7FA563),
    error: HexColor::new(0xD8647E),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#1c1c24",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Vague: [`VAGUE_LIGHT`] (derived) paired with [`VAGUE_DARK`].
    pub const VAGUE: ThemeSet = ThemeSet::pair("Vague", &VAGUE_LIGHT, &VAGUE_DARK);
}
