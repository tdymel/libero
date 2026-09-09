//! Gruvbox - Gruvbox Material dark, Gruvbox Light Soft light; Gruvbox
//! Classic, `morhetz/gruvbox` at medium contrast in both schemes; and Gruvbox
//! Soft, the pack's Dark Soft paired with the same Light Soft.
//!
//! Every dark half is the Kopuz pack's. Gruvbox Classic's light half is
//! ported from `colors/gruvbox.vim`, which gives each scheme the same slots
//! (`bg0`, `bg1`, `fg1`, `fg4`, blue, purple, green, red) and swaps the
//! bright accents for the faded ones on a light background.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

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
    ..Theme::DARK
};

/// Gruvbox Classic light, `morhetz/gruvbox` at medium contrast: the slots of
/// [`GRUVBOX_CLASSIC_DARK`], with the faded accents gruvbox.vim swaps in for
/// a light background (the neutral blue is shared).
pub static GRUVBOX_CLASSIC_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFBF1C7),
    ink: HexColor::new(0x3C3836),
    neutral: HexColor::new(0x3C3836),
    muted: HexColor::new(0x7C6F64),
    primary: HexColor::new(0x458588),
    secondary: HexColor::new(0x8F3F71),
    info: HexColor::new(0x076678),
    success: HexColor::new(0x79740E),
    error: HexColor::new(0x9D0006),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#ebdbb2",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Gruvbox Classic, `morhetz/gruvbox` dark at medium contrast.
pub static GRUVBOX_CLASSIC_DARK: Theme = Theme {
    surface: HexColor::new(0x282828),
    ink: HexColor::new(0xEBDBB2),
    neutral: HexColor::new(0xEBDBB2),
    muted: HexColor::new(0xA89984),
    primary: HexColor::new(0x458588),
    secondary: HexColor::new(0xD3869B),
    info: HexColor::new(0x83A598),
    success: HexColor::new(0xB8BB26),
    error: HexColor::new(0xFB4934),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#3c3836",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

/// Gruvbox Dark Soft.
pub static GRUVBOX_SOFT_DARK: Theme = Theme {
    surface: HexColor::new(0x32302F),
    ink: HexColor::new(0xFBF1C7),
    neutral: HexColor::new(0xFBF1C7),
    muted: HexColor::new(0xA89984),
    primary: HexColor::new(0x83A598),
    secondary: HexColor::new(0xD3869B),
    info: HexColor::new(0x8EC07C),
    success: HexColor::new(0xB8BB26),
    error: HexColor::new(0xFB4934),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#3c3836",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Gruvbox: [`GRUVBOX_LIGHT`] paired with [`GRUVBOX_DARK`].
    pub const GRUVBOX: ThemeSet = ThemeSet::pair("Gruvbox", &GRUVBOX_LIGHT, &GRUVBOX_DARK);

    /// Gruvbox Classic: [`GRUVBOX_CLASSIC_LIGHT`] paired with
    /// [`GRUVBOX_CLASSIC_DARK`].
    pub const GRUVBOX_CLASSIC: ThemeSet = ThemeSet::pair(
        "Gruvbox Classic",
        &GRUVBOX_CLASSIC_LIGHT,
        &GRUVBOX_CLASSIC_DARK,
    );

    /// Gruvbox Soft: [`GRUVBOX_LIGHT`] (Gruvbox Light Soft) paired with
    /// [`GRUVBOX_SOFT_DARK`].
    pub const GRUVBOX_SOFT: ThemeSet =
        ThemeSet::pair("Gruvbox Soft", &GRUVBOX_LIGHT, &GRUVBOX_SOFT_DARK);
}
