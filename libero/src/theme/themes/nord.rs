//! Nord: the Kopuz pack's dark, and the bright ambiance Nord's docs describe
//! (<https://www.nordtheme.com/docs/colors-and-palettes>) as Helix ships it.
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

/// Nord, bright ambiance. `primary`, `secondary`, `info` and `success` are Nord's `88C0D0`,
/// `B48EAD`, `8FBCBB` and `A3BE8C` darkened, `warning` is Libero's amber `FAB005` darkened,
/// hue kept, until their text role reads at 4.5:1 (todos 2034, 3236).
pub static NORD_LIGHT: Theme = Theme {
    surface: HexColor::new(0xECEFF4),
    ink: HexColor::new(0x2E3440),
    neutral: HexColor::new(0x2E3440),
    muted: HexColor::new(0x4C566A),
    primary: HexColor::new(0x6B98A4),
    secondary: HexColor::new(0xA985A3),
    warning: HexColor::new(0xBE8604),
    info: HexColor::new(0x729696),
    success: HexColor::new(0x829870),
    error: HexColor::new(0xBF616A),
    paper: PaperDefaults {
        background: "#d8dee9",
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    gradient: GRADIENT,
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
    paper: PaperDefaults {
        background: "#3b4252",
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    gradient: GRADIENT,
    ..Theme::DARK
};

impl ThemeSet {
    /// Nord: [`NORD_LIGHT`] paired with [`NORD_DARK`].
    pub const NORD: ThemeSet = ThemeSet::pair("Nord", &NORD_LIGHT, &NORD_DARK);
}
