//! GitHub - Primer's light and dark (default) themes, from
//! `primer/primitives` (`@primer/primitives` 11.10.0, functional tokens).

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Primer light.
pub static GITHUB_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFFFFFF),
    ink: HexColor::new(0x1F2328),
    neutral: HexColor::new(0x1F2328),
    muted: HexColor::new(0x59636E),
    primary: HexColor::new(0x0969DA),
    secondary: HexColor::new(0x8250DF),
    info: HexColor::new(0x006A80),
    success: HexColor::new(0x1A7F37),
    error: HexColor::new(0xD1242F),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#f6f8fa",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Primer dark (default).
pub static GITHUB_DARK: Theme = Theme {
    surface: HexColor::new(0x0D1117),
    ink: HexColor::new(0xF0F6FC),
    neutral: HexColor::new(0xF0F6FC),
    muted: HexColor::new(0x9198A1),
    primary: HexColor::new(0x4493F8),
    secondary: HexColor::new(0xAB7DF8),
    info: HexColor::new(0x07ACE4),
    success: HexColor::new(0x3FB950),
    error: HexColor::new(0xF85149),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#151b23",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// GitHub: [`GITHUB_LIGHT`] paired with [`GITHUB_DARK`].
    pub const GITHUB: ThemeSet = ThemeSet::pair("GitHub", &GITHUB_LIGHT, &GITHUB_DARK);
}
