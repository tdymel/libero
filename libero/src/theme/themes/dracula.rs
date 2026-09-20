//! Dracula - Alucard light, Dracula dark.
//!
//! Alucard is mapped slot for slot from <https://draculatheme.com/spec>; the spec has
//! no `accent-soft`, so `info` takes Alucard's Cyan.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Alucard.
pub static DRACULA_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFFFBEB),
    ink: HexColor::new(0x1F1F1F),
    neutral: HexColor::new(0x1F1F1F),
    muted: HexColor::new(0x6C664B),
    primary: HexColor::new(0x644AC9),
    secondary: HexColor::new(0xA3144D),
    info: HexColor::new(0x036A96),
    success: HexColor::new(0x14710A),
    error: HexColor::new(0xCB3A2A),
    paper: PaperDefaults {
        background: "#ceccc0",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Dracula.
pub static DRACULA_DARK: Theme = Theme {
    surface: HexColor::new(0x282A36),
    ink: HexColor::new(0xF8F8F2),
    neutral: HexColor::new(0xF8F8F2),
    muted: HexColor::new(0x6272A4),
    primary: HexColor::new(0xBD93F9),
    secondary: HexColor::new(0xFF79C6),
    info: HexColor::new(0xCAA7FC),
    success: HexColor::new(0x50FA7B),
    error: HexColor::new(0xFF5555),
    paper: PaperDefaults {
        background: "#21222c",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Dracula: [`DRACULA_LIGHT`] (Alucard) paired with [`DRACULA_DARK`].
    pub const DRACULA: ThemeSet = ThemeSet::pair("Dracula", &DRACULA_LIGHT, &DRACULA_DARK);
}
