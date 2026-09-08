//! Kanagawa - Lotus light and Wave dark, from `rebelot/kanagawa.nvim`.

use super::super::{
    CodeBlockDefaults, CodeDefaults, HexColor, KbdDefaults, PaperDefaults, Theme, ThemeSet,
    TooltipDefaults,
};

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
    code_block: CodeBlockDefaults::DEFAULT,
    kbd: KbdDefaults::DEFAULT,
    tooltip: TooltipDefaults::DEFAULT,
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
    code_block: CodeBlockDefaults::DARK,
    kbd: KbdDefaults::DARK,
    tooltip: TooltipDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Kanagawa: [`KANAGAWA_LIGHT`] paired with [`KANAGAWA_DARK`].
    pub const KANAGAWA: ThemeSet = ThemeSet::pair("Kanagawa", &KANAGAWA_LIGHT, &KANAGAWA_DARK);
}
