//! Gruvbox - Gruvbox Material dark, Gruvbox Light Soft light.

use super::super::{
    CodeBlockDefaults, CodeDefaults, HexColor, KbdDefaults, PaperDefaults, Theme, ThemeSet,
    TooltipDefaults,
};

/// Gruvbox Light Soft.
pub static GRUVBOX_LIGHT: Theme = Theme {
    surface: HexColor::new(0xF2E5BC),
    ink: HexColor::new(0x3C3836),
    neutral: HexColor::new(0x3C3836),
    muted: HexColor::new(0x665C54),
    primary: HexColor::new(0x458588),
    secondary: HexColor::new(0x8F3F71),
    info: HexColor::new(0x076678),
    success: HexColor::new(0x79740E),
    error: HexColor::new(0xCC241D),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#f9f5d7",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    code_block: CodeBlockDefaults::DEFAULT,
    kbd: KbdDefaults::DEFAULT,
    tooltip: TooltipDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Gruvbox Material.
pub static GRUVBOX_DARK: Theme = Theme {
    surface: HexColor::new(0x1D2021),
    ink: HexColor::new(0xD4BE98),
    neutral: HexColor::new(0xD4BE98),
    muted: HexColor::new(0xA89984),
    primary: HexColor::new(0x7DAEA3),
    secondary: HexColor::new(0xD3869B),
    info: HexColor::new(0x89B482),
    success: HexColor::new(0xA9B665),
    error: HexColor::new(0xEA6962),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#282828",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    code_block: CodeBlockDefaults::DARK,
    kbd: KbdDefaults::DARK,
    tooltip: TooltipDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Gruvbox: [`GRUVBOX_LIGHT`] paired with [`GRUVBOX_DARK`].
    pub const GRUVBOX: ThemeSet = ThemeSet::pair("Gruvbox", &GRUVBOX_LIGHT, &GRUVBOX_DARK);
}
