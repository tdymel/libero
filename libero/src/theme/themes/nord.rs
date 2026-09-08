//! Nord - Nord's bright ambiance light, Nord dark.
//!
//! The dark half is the Kopuz pack's. Nord has no separate light theme: its
//! own documentation (<https://www.nordtheme.com/docs/colors-and-palettes>)
//! describes a bright ambiance built from the same sixteen colours - Snow
//! Storm `nord6` as the background, Polar Night `nord0` for plain text,
//! `nord3` for the most subtle UI text, `nord4` for elevated panels and
//! popups - with Frost and Aurora unchanged. The light half is that, as
//! Helix ships it built in (`runtime/themes/nord_light.toml`, card `nord4`).
//! The most-installed VS Code "Nord Light" (huytd) was not taken: its chrome
//! and syntax are GitHub Light's colours on Nord's backgrounds.

use super::super::{
    CodeBlockDefaults, CodeDefaults, HexColor, KbdDefaults, PaperDefaults, Theme, ThemeSet,
    TooltipDefaults,
};

/// Nord, bright ambiance.
pub static NORD_LIGHT: Theme = Theme {
    surface: HexColor::new(0xECEFF4),
    ink: HexColor::new(0x2E3440),
    neutral: HexColor::new(0x2E3440),
    muted: HexColor::new(0x4C566A),
    primary: HexColor::new(0x88C0D0),
    secondary: HexColor::new(0xB48EAD),
    info: HexColor::new(0x8FBCBB),
    success: HexColor::new(0xA3BE8C),
    error: HexColor::new(0xBF616A),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#d8dee9",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    code_block: CodeBlockDefaults::DEFAULT,
    kbd: KbdDefaults::DEFAULT,
    tooltip: TooltipDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// Nord.
pub static NORD_DARK: Theme = Theme {
    surface: HexColor::new(0x2E3440),
    ink: HexColor::new(0xD8DEE9),
    neutral: HexColor::new(0xD8DEE9),
    muted: HexColor::new(0x81A1C1),
    primary: HexColor::new(0x88C0D0),
    secondary: HexColor::new(0xB48EAD),
    info: HexColor::new(0x8FBCBB),
    success: HexColor::new(0xA3BE8C),
    error: HexColor::new(0xBF616A),
    // Warning stays Libero's amber, as in every ported theme.
    paper: PaperDefaults {
        background: "#3b4252",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    code_block: CodeBlockDefaults::DARK,
    kbd: KbdDefaults::DARK,
    tooltip: TooltipDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// Nord: [`NORD_LIGHT`] paired with [`NORD_DARK`].
    pub const NORD: ThemeSet = ThemeSet::pair("Nord", &NORD_LIGHT, &NORD_DARK);
}
