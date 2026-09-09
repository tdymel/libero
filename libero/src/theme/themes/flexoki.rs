//! Flexoki - Flexoki light and dark, from `kepano/flexoki`.
//!
//! The UI mapping is upstream's own (stephango.com/flexoki): `bg`, `bg-2`,
//! `tx`, `tx-2`, with the 600 accents on light and the 400s on dark.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet, TooltipDefaults};

/// Flexoki light: `paper` page, `base-50` card, `black` text.
pub static FLEXOKI_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFFFCF0),
    ink: HexColor::new(0x100F0F),
    neutral: HexColor::new(0x100F0F),
    muted: HexColor::new(0x6F6E69),
    primary: HexColor::new(0x205EA6),
    secondary: HexColor::new(0x5E409D),
    info: HexColor::new(0x24837B),
    success: HexColor::new(0x66800B),
    error: HexColor::new(0xAF3029),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#f2f0e5",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    tooltip: TooltipDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Flexoki dark: `black` page, `base-950` card, `base-200` text.
pub static FLEXOKI_DARK: Theme = Theme {
    surface: HexColor::new(0x100F0F),
    ink: HexColor::new(0xCECDC3),
    neutral: HexColor::new(0xCECDC3),
    muted: HexColor::new(0x878580),
    primary: HexColor::new(0x4385BE),
    secondary: HexColor::new(0x8B7EC8),
    info: HexColor::new(0x3AA99F),
    success: HexColor::new(0x879A39),
    error: HexColor::new(0xD14D41),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#1c1b1a",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    tooltip: TooltipDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Flexoki: [`FLEXOKI_LIGHT`] paired with [`FLEXOKI_DARK`].
    pub const FLEXOKI: ThemeSet = ThemeSet::pair("Flexoki", &FLEXOKI_LIGHT, &FLEXOKI_DARK);
}
