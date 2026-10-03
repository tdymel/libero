//! Vague - a derived light half, and Vague dark (`vague-theme/vague.nvim`).
//!
//! Dark-only upstream (vague.nvim issue 90), so the light half is ours, by the rule
//! Osmium and kettek16 share:
//!
//! - the five accents are the dark half's; the stylesheet re-derives roles per page;
//! - `ink` is the dark `bg`, `#141415`;
//! - page and card are the dark `text` (`#cdcdcd`) mixed 80% and 60% toward white;
//! - `muted` is the dark `text-muted` (`#606079`) moved to 3.03:1 from the page,
//!   the first step past 3:1.
//!
//! The gradient runs info to secondary: no label reads at 4.5:1 on primary to secondary
//! (todo 2051).

use super::super::{
    CodeDefaults, Color, GradientDefaults, HexColor, PaperDefaults, Theme, ThemeSet,
};

const GRADIENT: GradientDefaults = GradientDefaults {
    from: Color::Info,
    ..GradientDefaults::DEFAULT
};

/// Vague, light - derived, not upstream.
pub static VAGUE_LIGHT: Theme = Theme {
    surface: HexColor::new(0xF5F5F5),
    ink: HexColor::new(0x141415),
    neutral: HexColor::new(0x141415),
    muted: HexColor::new(0x8C8C9E),
    primary: HexColor::new(0x6E94B2),
    secondary: HexColor::new(0xBB9DBD),
    info: HexColor::new(0x7E98E8),
    success: HexColor::new(0x7FA563),
    error: HexColor::new(0xD8647E),
    paper: PaperDefaults {
        background: "#ebebeb",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    gradient: GRADIENT,
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
    paper: PaperDefaults {
        background: "#1c1c24",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    gradient: GRADIENT,
    ..Theme::DARK
};

impl ThemeSet {
    /// Vague: [`VAGUE_LIGHT`] (derived) paired with [`VAGUE_DARK`].
    pub const VAGUE: ThemeSet = ThemeSet::pair("Vague", &VAGUE_LIGHT, &VAGUE_DARK);
}
