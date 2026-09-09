//! kettek16 - a derived light half, and kettek16 dark.
//!
//! The dark half is the Kopuz pack's; upstream is `kettek/vscode-colorblind16`,
//! a colour-blind-friendly theme whose accents are Paul Tol's "vibrant" and
//! "bright" qualitative colours. It is dark-only and no community light
//! variant exists, so **the light half is ours**, derived by the rule Vague
//! and Osmium share (the module doc of [`super`] states it once):
//!
//! - the five accents are the dark half's own - Tol drew them for a white
//!   page in the first place;
//! - `ink` is the dark `bg`, `#090909`;
//! - the page is the dark `text` (`#fef638`, kettek16's yellow) mixed 80%
//!   toward white, the card the same mixed 60%, so the yellow survives as a
//!   pale cream;
//! - `muted` is the dark `text-muted` (`#dddddd`) moved toward the ink until
//!   it is as far from the light page (14.66:1) as it is from the dark one -
//!   kettek16's quiet text is barely quieter than its ink, and stays so.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Theme, ThemeSet};

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
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#fffbaf",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
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
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#1d1f21",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// kettek16: [`KETTEK16_LIGHT`] (derived) paired with [`KETTEK16_DARK`].
    pub const KETTEK16: ThemeSet = ThemeSet::pair("kettek16", &KETTEK16_LIGHT, &KETTEK16_DARK);
}
