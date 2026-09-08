//! Catppuccin - Catppuccin Mocha and Latte.

use super::super::{
    CodeBlockDefaults, CodeDefaults, HexColor, KbdDefaults, PaperDefaults, Theme, ThemeSet,
    TooltipDefaults,
};

/// Catppuccin Latte.
pub static CATPPUCCIN_LIGHT: Theme = Theme {
    surface: HexColor::new(0xEFF1F5),
    ink: HexColor::new(0x4C4F69),
    neutral: HexColor::new(0x4C4F69),
    muted: HexColor::new(0x6C6F85),
    primary: HexColor::new(0x1E66F5),
    secondary: HexColor::new(0x8839EF),
    info: HexColor::new(0x209FB5),
    success: HexColor::new(0x40A02B),
    error: HexColor::new(0xD20F39),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#e6e9ef",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    code_block: CodeBlockDefaults::DEFAULT,
    kbd: KbdDefaults::DEFAULT,
    tooltip: TooltipDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Catppuccin Mocha.
pub static CATPPUCCIN_DARK: Theme = Theme {
    surface: HexColor::new(0x1E1E2E),
    ink: HexColor::new(0xCDD6F4),
    neutral: HexColor::new(0xCDD6F4),
    muted: HexColor::new(0xBAC2DE),
    primary: HexColor::new(0x89B4FA),
    secondary: HexColor::new(0xCBA6F7),
    info: HexColor::new(0x89DCEB),
    success: HexColor::new(0xA6E3A1),
    error: HexColor::new(0xF38BA8),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#181825",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    code_block: CodeBlockDefaults::DARK,
    kbd: KbdDefaults::DARK,
    tooltip: TooltipDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Catppuccin: [`CATPPUCCIN_LIGHT`] paired with [`CATPPUCCIN_DARK`].
    pub const CATPPUCCIN: ThemeSet =
        ThemeSet::pair("Catppuccin", &CATPPUCCIN_LIGHT, &CATPPUCCIN_DARK);
}
