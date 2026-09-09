//! Osmium - a derived light half, and IroncladDev's Osmium dark.
//!
//! The dark half is the Kopuz pack's, which matches the upstream palette
//! (`@webtui/theme-osmium`: `root`, `foreground0`, `foreground1`,
//! `purple-fg`, `pink-fg`, `blue-fg`, `green-fg`, `red-fg`, `surface0`).
//! Osmium is dark-only upstream and no community light variant exists, so
//! **the light half is ours**, derived by the rule Vague and kettek16 share
//! (the module doc of [`super`] states it once):
//!
//! - the five accents are the dark half's own;
//! - `ink` is the dark `bg`, `#14131e`;
//! - the page is the dark `text` (`#c8d5f1`) mixed 80% toward white, the card
//!   the same mixed 60%;
//! - `muted` is the dark `text-muted` (`#949bb9`) moved toward the ink until
//!   it is as far from the light page (6.70:1) as it is from the dark one.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet, TooltipDefaults};

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
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#e9eef9",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    tooltip: TooltipDefaults::DEFAULT,
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
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#1f1d2d",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    tooltip: TooltipDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Osmium: [`OSMIUM_LIGHT`] (derived) paired with [`OSMIUM_DARK`].
    pub const OSMIUM: ThemeSet = ThemeSet::pair("Osmium", &OSMIUM_LIGHT, &OSMIUM_DARK);
}
