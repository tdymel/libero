//! shadcn/ui - the neutral base colour of its published theme (`--radius: 0.625rem`),
//! oklch converted to sRGB hex. Only tokens with a role of ours are mapped; `secondary`
//! is the theme's `chart-3` and `chart-1`, as shadcn's own `secondary` is a pale surface.

use super::super::{CodeDefaults, HexColor, PaperDefaults, Size, Sizes, Theme, ThemeSet};

/// `--radius` is 10px, which is `lg`; shadcn's `sm` and `md` are 6px and 8px, its `xl`
/// 14px. `xxl` stays the pill our badges and indicators expect.
const RADIUS: Sizes<u8> = Sizes::new(4, 6, 8, 10, 14, 64);

/// shadcn neutral, light.
pub static SHADCN_LIGHT: Theme = Theme {
    radius: RADIUS,
    surface: HexColor::new(0xFFFFFF),
    ink: HexColor::new(0x0A0A0A),
    neutral: HexColor::new(0x0A0A0A),
    muted: HexColor::new(0x737373),
    primary: HexColor::new(0x171717),
    secondary: HexColor::new(0x525252),
    error: HexColor::new(0xE7000B),
    paper: PaperDefaults {
        background: "#ffffff",
        radius: Size::Xl,
        ..PaperDefaults::DEFAULT
    },
    code: CodeDefaults::DEFAULT,
    ..Theme::DEFAULT
};

/// shadcn neutral, dark.
pub static SHADCN_DARK: Theme = Theme {
    radius: RADIUS,
    surface: HexColor::new(0x0A0A0A),
    ink: HexColor::new(0xFAFAFA),
    neutral: HexColor::new(0xFAFAFA),
    muted: HexColor::new(0xA1A1A1),
    primary: HexColor::new(0xE5E5E5),
    secondary: HexColor::new(0xD4D4D4),
    error: HexColor::new(0xFF6467),
    paper: PaperDefaults {
        background: "#171717",
        radius: Size::Xl,
        ..PaperDefaults::DARK
    },
    code: CodeDefaults::DARK,
    ..Theme::DARK
};

impl ThemeSet {
    /// shadcn/ui: [`SHADCN_LIGHT`] paired with [`SHADCN_DARK`].
    pub const SHADCN: ThemeSet = ThemeSet::pair("shadcn/ui", &SHADCN_LIGHT, &SHADCN_DARK);
}
