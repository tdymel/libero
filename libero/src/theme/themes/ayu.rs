//! Ayu - Ayu Dark and Ayu Light.

use super::super::{
    CodeBlockDefaults, CodeDefaults, HexColor, KbdDefaults, PaperDefaults, Theme, ThemeSet,
    TooltipDefaults,
};

/// Ayu Light.
pub static AYU_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFAFAFA),
    ink: HexColor::new(0x5C6166),
    neutral: HexColor::new(0x5C6166),
    muted: HexColor::new(0x8A9199),
    primary: HexColor::new(0x399EE6),
    secondary: HexColor::new(0xA37ACC),
    info: HexColor::new(0x55B4D4),
    success: HexColor::new(0x86B300),
    error: HexColor::new(0xF07171),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#f3f3f3",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    code_block: CodeBlockDefaults::DEFAULT,
    kbd: KbdDefaults::DEFAULT,
    tooltip: TooltipDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Ayu Dark.
pub static AYU_DARK: Theme = Theme {
    surface: HexColor::new(0x0A0E14),
    ink: HexColor::new(0xB3B1AD),
    neutral: HexColor::new(0xB3B1AD),
    muted: HexColor::new(0x626A73),
    primary: HexColor::new(0x39BAE6),
    secondary: HexColor::new(0xD2A6FF),
    info: HexColor::new(0x59C2FF),
    success: HexColor::new(0xC2D94C),
    error: HexColor::new(0xF07178),
    // No warning colour in the palette, so Libero's amber stands in.
    paper: PaperDefaults {
        background: "#0d1017",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    code_block: CodeBlockDefaults::DARK,
    kbd: KbdDefaults::DARK,
    tooltip: TooltipDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Ayu: [`AYU_LIGHT`] paired with [`AYU_DARK`].
    pub const AYU: ThemeSet = ThemeSet::pair("Ayu", &AYU_LIGHT, &AYU_DARK);
}
