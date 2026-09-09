//! Kanagawa - Lotus light and Wave dark, from `rebelot/kanagawa.nvim`, and
//! Kanagawa Dragon, whose dark half is the Kopuz pack's and whose light half
//! is Lotus again.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Kanagawa Lotus.
pub static KANAGAWA_LIGHT: Theme = Theme {
    surface: HexColor::new(0xF2ECBC),
    ink: HexColor::new(0x545464),
    neutral: HexColor::new(0x545464),
    muted: HexColor::new(0x8A8980),
    primary: HexColor::new(0x4D699B),
    secondary: HexColor::new(0x624C83),
    info: HexColor::new(0x4E8CA2),
    success: HexColor::new(0x6F894E),
    error: HexColor::new(0xE82424),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#d5cea3",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Kanagawa Wave.
pub static KANAGAWA_DARK: Theme = Theme {
    surface: HexColor::new(0x1F1F28),
    ink: HexColor::new(0xDCD7BA),
    neutral: HexColor::new(0xDCD7BA),
    muted: HexColor::new(0x727169),
    primary: HexColor::new(0x7E9CD8),
    secondary: HexColor::new(0x957FB8),
    info: HexColor::new(0x7FB4CA),
    success: HexColor::new(0x98BB6C),
    error: HexColor::new(0xE82424),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#16161d",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

/// Kanagawa Dragon, from the Kopuz pack.
pub static KANAGAWA_DRAGON_DARK: Theme = Theme {
    surface: HexColor::new(0x181616),
    ink: HexColor::new(0xC5C9C5),
    neutral: HexColor::new(0xC5C9C5),
    muted: HexColor::new(0xA6A69C),
    primary: HexColor::new(0x949FB5),
    secondary: HexColor::new(0xA292A3),
    info: HexColor::new(0x8BA4B0),
    success: HexColor::new(0x87A987),
    error: HexColor::new(0xC4746E),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#1d1c19",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Kanagawa: [`KANAGAWA_LIGHT`] paired with [`KANAGAWA_DARK`].
    pub const KANAGAWA: ThemeSet = ThemeSet::pair("Kanagawa", &KANAGAWA_LIGHT, &KANAGAWA_DARK);

    /// Kanagawa Dragon: [`KANAGAWA_LIGHT`] paired with
    /// [`KANAGAWA_DRAGON_DARK`]. Lotus is kanagawa.nvim's one light theme,
    /// the light half of Wave and Dragon alike.
    pub const KANAGAWA_DRAGON: ThemeSet =
        ThemeSet::pair("Kanagawa Dragon", &KANAGAWA_LIGHT, &KANAGAWA_DRAGON_DARK);
}
