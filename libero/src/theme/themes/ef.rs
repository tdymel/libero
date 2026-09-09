//! Ef Night - ef-day light, ef-night dark, from `protesilaos/ef-themes`.
//!
//! The dark half is the Kopuz pack's, which is `ef-night-theme.el`'s
//! palette: `bg-main`, `fg-main`, `fg-dim`, `blue`, `magenta-cooler`,
//! `cyan-warmer`, `green`, `red`, and `bg-dim` for the card. The light half
//! is the same slots of `ef-day-theme.el`. Ef-themes does not document any
//! two of its themes as a pair; ef-day is ef-night's counterpart by name
//! only.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

/// Ef Day.
pub static EF_NIGHT_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFFF5EA),
    ink: HexColor::new(0x584141),
    neutral: HexColor::new(0x584141),
    muted: HexColor::new(0x63728F),
    primary: HexColor::new(0x375CC6),
    secondary: HexColor::new(0x8448AA),
    info: HexColor::new(0x3F6FAF),
    success: HexColor::new(0x007A0A),
    error: HexColor::new(0xBA2D2F),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#f2e9db",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Ef Night.
pub static EF_NIGHT_DARK: Theme = Theme {
    surface: HexColor::new(0x000E17),
    ink: HexColor::new(0xAFBCBF),
    neutral: HexColor::new(0xAFBCBF),
    muted: HexColor::new(0x70819F),
    primary: HexColor::new(0x379CF6),
    secondary: HexColor::new(0xAF8AFF),
    info: HexColor::new(0x6FAFFF),
    success: HexColor::new(0x1FA526),
    error: HexColor::new(0xEF656A),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#1a202b",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Ef Night: [`EF_NIGHT_LIGHT`] (ef-day) paired with [`EF_NIGHT_DARK`].
    pub const EF_NIGHT: ThemeSet = ThemeSet::pair("Ef Night", &EF_NIGHT_LIGHT, &EF_NIGHT_DARK);
}
