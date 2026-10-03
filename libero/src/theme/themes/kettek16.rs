//! kettek16 - a derived light half, and kettek16 dark.
//!
//! Upstream is `kettek/vscode-colorblind16` (Paul Tol's accents, drawn for white). Dark-only,
//! so the light half is ours, by the rule in [`vague`](super::VAGUE_LIGHT):
//!
//! - `ink` is the dark `bg`, `#090909`; page and card are `#fef638` mixed 80%/60% to white;
//! - `muted` is `#dddddd` moved toward the ink to the same 14.66:1 from the page.
//!
//! The gradient runs primary to info: no label reads at 4.5:1 on primary to secondary
//! (todo 2051).

use super::super::{
    CodeDefaults, Color, GradientDefaults, HexColor, PaperDefaults, Theme, ThemeSet,
};

const GRADIENT: GradientDefaults = GradientDefaults {
    to: Color::Info,
    ..GradientDefaults::DEFAULT
};

/// kettek16, light - derived, not upstream.
pub static KETTEK16_LIGHT: Theme = Theme {
    surface: HexColor::new(0xFFFDD7),
    ink: HexColor::new(0x090909),
    neutral: HexColor::new(0x090909),
    muted: HexColor::new(0x252525),
    primary: HexColor::new(0x33BBEE),
    secondary: HexColor::new(0xEE3377),
    info: HexColor::new(0x66CCEE),
    success: HexColor::new(0x009988),
    error: HexColor::new(0xEE6677),
    paper: PaperDefaults {
        background: "#fffbaf",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    gradient: GRADIENT,
    ..Theme::DEFAULT
};

/// kettek16.
pub static KETTEK16_DARK: Theme = Theme {
    surface: HexColor::new(0x090909),
    ink: HexColor::new(0xFEF638),
    neutral: HexColor::new(0xFEF638),
    muted: HexColor::new(0xDDDDDD),
    primary: HexColor::new(0x33BBEE),
    secondary: HexColor::new(0xEE3377),
    info: HexColor::new(0x66CCEE),
    success: HexColor::new(0x009988),
    error: HexColor::new(0xEE6677),
    paper: PaperDefaults {
        background: "#1d1f21",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    gradient: GRADIENT,
    ..Theme::DARK
};

impl ThemeSet {
    /// kettek16: [`KETTEK16_LIGHT`] (derived) paired with [`KETTEK16_DARK`].
    pub const KETTEK16: ThemeSet = ThemeSet::pair("kettek16", &KETTEK16_LIGHT, &KETTEK16_DARK);
}
