use std::fmt::{self, Display};

use super::{ColorShade, ShadeRamp};

/// WCAG 1.4.3 for small text. Every text or fill role is held to it: a component
/// can't know its text size.
pub(crate) const TEXT_CONTRAST: f32 = 4.5;

/// A theme's `surface` and `ink`: neutral ramps mix between them, roles measure against them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ends {
    pub(crate) surface: HexColor,
    pub(crate) ink: HexColor,
}

impl Ends {
    /// The end a black or white `pick` is painted as (on a dark theme, white is the ink).
    /// Measure foregrounds against this, not pure black/white, or it flatters the pair.
    pub(crate) fn foreground(self, pick: HexColor) -> HexColor {
        if self.foreground_is_ink(pick) {
            self.ink
        } else {
            self.surface
        }
    }

    /// The same choice, for a caller that names the var instead of the hex.
    pub(crate) fn foreground_is_ink(self, pick: HexColor) -> bool {
        let wants_dark = pick.rgb() == 0x00_00_00;
        let ink_is_dark = self.ink.contrast().rgb() == 0xFF_FF_FF;
        wants_dark == ink_is_dark
    }

    /// Pure white and black: the tests' stand-in for the default light theme.
    #[cfg(test)]
    pub(crate) const ABSOLUTE: Ends = Ends {
        surface: HexColor::new(0xFF_FF_FF),
        ink: HexColor::new(0x00_00_00),
    };
}

/// An opaque RGB colour, e.g. a theme's palette base.
///
/// ```
/// # use libero::theme::HexColor;
/// const BRAND: HexColor = HexColor::new(0x22_8B_E6);
/// assert_eq!(BRAND.to_string(), "#228BE6");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HexColor {
    rgb: u32,
}

impl HexColor {
    pub const fn new(rgb: u32) -> Self {
        if rgb > 0xFF_FF_FF {
            panic!("hex color out of range");
        }
        Self { rgb }
    }

    /// `#rgb`/`#rrggbb`, `rgb()`, opaque `rgba()`. `None` for a translucent `rgba()`
    /// (its look depends on what shows through), names, `hsl()` and vars.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        let value = value.trim();

        if let Some(hex) = value.strip_prefix('#') {
            return Self::parse_hex(hex);
        }
        if let Some(inner) = value
            .strip_prefix("rgba(")
            .and_then(|s| s.strip_suffix(')'))
        {
            return Self::parse_rgb_channels(inner, true);
        }
        if let Some(inner) = value.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            return Self::parse_rgb_channels(inner, false);
        }

        None
    }

    fn parse_hex(hex: &str) -> Option<Self> {
        let expand = |c: char| c.to_digit(16).map(|d| (d * 16 + d) as u8);

        match hex.len() {
            3 => {
                let mut chars = hex.chars();
                let r = expand(chars.next()?)?;
                let g = expand(chars.next()?)?;
                let b = expand(chars.next()?)?;
                Some(Self::new(((r as u32) << 16) | ((g as u32) << 8) | b as u32))
            }
            6 => u32::from_str_radix(hex, 16).ok().map(Self::new),
            _ => None,
        }
    }

    fn parse_rgb_channels(inner: &str, has_alpha: bool) -> Option<Self> {
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() != if has_alpha { 4 } else { 3 } {
            return None;
        }

        if has_alpha {
            let alpha: f32 = parts[3].parse().ok()?;
            if alpha < 1.0 {
                return None;
            }
        }

        let r: u8 = parts[0].parse().ok()?;
        let g: u8 = parts[1].parse().ok()?;
        let b: u8 = parts[2].parse().ok()?;
        Some(Self::new(((r as u32) << 16) | ((g as u32) << 8) | b as u32))
    }

    pub const fn r(self) -> u8 {
        ((self.rgb >> 16) & 0xFF) as u8
    }

    pub const fn g(self) -> u8 {
        ((self.rgb >> 8) & 0xFF) as u8
    }

    pub const fn b(self) -> u8 {
        (self.rgb & 0xFF) as u8
    }

    pub const fn rgb(self) -> u32 {
        self.rgb
    }

    /// A ramp step as distance from the page: S1 barely leaves the surface, S9 is furthest.
    /// Both start at the surface; see [`far_end`] for where they end (todo 69).
    pub(crate) const fn shade(self, shade: ColorShade, ramp: ShadeRamp, ends: Ends) -> Self {
        let percent = mix_percent(ramp, shade);
        let far = far_end(ramp, ends);

        if percent >= 0 {
            self.mix(ends.surface, percent as u8)
        } else {
            self.mix(far, (-percent) as u8)
        }
    }

    /// Black or white by perceived brightness: which foreground belongs, not whether it
    /// passes ([`contrast_ratio`](Self::contrast_ratio) says that).
    pub(crate) const fn contrast(self) -> Self {
        if self.luminance() >= 140 {
            HexColor::new(0x00_00_00)
        } else {
            HexColor::new(0xFF_FF_FF)
        }
    }

    /// The `color-scheme` of a surface this colour, so native controls match the theme.
    pub(crate) const fn color_scheme(self) -> &'static str {
        if self.contrast().rgb() == 0xFF_FF_FF {
            "dark"
        } else {
            "light"
        }
    }

    /// WCAG 2.x relative luminance; [`luminance`](Self::luminance) is only a cheap approximation.
    pub(crate) fn relative_luminance(self) -> f32 {
        fn channel(value: u8) -> f32 {
            let value = value as f32 / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }

        0.2126 * channel(self.r()) + 0.7152 * channel(self.g()) + 0.0722 * channel(self.b())
    }

    /// WCAG 2.x contrast ratio, 1.0 to 21.0. Symmetric.
    pub(crate) fn contrast_ratio(self, other: Self) -> f32 {
        let (a, b) = (self.relative_luminance(), other.relative_luminance());
        let (lighter, darker) = if a >= b { (a, b) } else { (b, a) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// [`contrast`](Self::contrast)'s pick, unless it fails [`TEXT_CONTRAST`] and the other
    /// does better: for mid steps, where brightness and the WCAG curve disagree.
    pub(crate) fn readable_contrast(self, ends: Ends) -> Self {
        let picked = self.contrast();
        if self.contrast_ratio(ends.foreground(picked)) >= TEXT_CONTRAST {
            return picked;
        }

        let other = if picked.rgb() == 0x00_00_00 {
            HexColor::new(0xFF_FF_FF)
        } else {
            HexColor::new(0x00_00_00)
        };

        if self.contrast_ratio(ends.foreground(other))
            > self.contrast_ratio(ends.foreground(picked))
        {
            other
        } else {
            picked
        }
    }

    /// This base (shade 6) as text: the smallest mix towards the far end that passes
    /// [`TEXT_CONTRAST`] on the surface and every card (todo 314); a whole step overshoots.
    pub(crate) fn text_base(self, ramp: ShadeRamp, ends: Ends, cards: &[Self]) -> Self {
        self.first_mix(ramp, ends, |text| {
            text.contrast_ratio(ends.surface) >= TEXT_CONTRAST
                && cards
                    .iter()
                    .all(|&card| text.contrast_ratio(card) >= TEXT_CONTRAST)
        })
    }

    /// This base as a fill: the smallest mix away from the surface on which its painted
    /// foreground passes [`TEXT_CONTRAST`] (`blue.6` + white is only 3.56:1).
    pub(crate) fn fill_base(self, ramp: ShadeRamp, ends: Ends) -> Self {
        self.first_mix(ramp, ends, |fill| {
            fill.contrast_ratio(ends.foreground(fill.contrast())) >= TEXT_CONTRAST
        })
    }

    /// [`text_base`](Self::text_base) for any colour on other backgrounds, at `floor`.
    pub(crate) fn readable_on(self, backgrounds: &[Self], floor: f32, ends: Ends) -> Self {
        let far = far_end(ShadeRamp::Chromatic, ends);
        (0..=100)
            .map(|weight| self.mix(far, weight))
            .find(|&text| {
                backgrounds
                    .iter()
                    .all(|&background| text.contrast_ratio(background) >= floor)
            })
            .unwrap_or(far)
    }

    /// The first 1% mix step towards the far end, up to [`ColorShade::S9`], that `passes`.
    fn first_mix(self, ramp: ShadeRamp, ends: Ends, passes: impl Fn(Self) -> bool) -> Self {
        let far = far_end(ramp, ends);
        let deepest = self.shade(ColorShade::S9, ramp, ends);
        (0..=(-mix_percent(ramp, ColorShade::S9)) as u8)
            .map(|weight| self.mix(far, weight))
            .find(|&color| passes(color))
            .unwrap_or(deepest)
    }

    const fn mix(self, other: Self, weight: u8) -> Self {
        let r = mix_channel(self.r(), other.r(), weight);
        let g = mix_channel(self.g(), other.g(), weight);
        let b = mix_channel(self.b(), other.b(), weight);
        Self::new(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
    }

    const fn luminance(self) -> u16 {
        (self.r() as u16 * 30 + self.g() as u16 * 59 + self.b() as u16 * 11) / 100
    }
}

impl Display for HexColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r(), self.g(), self.b())
    }
}

/// How far each step of a ramp is mixed from its base, in percent: towards
/// the surface when positive, towards [`far_end`] when negative.
const fn mix_percent(ramp: ShadeRamp, shade: ColorShade) -> i8 {
    match ramp {
        ShadeRamp::Chromatic => match shade {
            ColorShade::S1 => 80,
            ColorShade::S2 => 62,
            ColorShade::S3 => 41,
            ColorShade::S4 => 22,
            ColorShade::S5 => 9,
            ColorShade::S6 => 0,
            ColorShade::S7 => -7,
            ColorShade::S8 => -16,
            ColorShade::S9 => -25,
        },
        ShadeRamp::Neutral => match shade {
            ColorShade::S1 => 90,
            ColorShade::S2 => 84,
            ColorShade::S3 => 75,
            ColorShade::S4 => 62,
            ColorShade::S5 => 35,
            ColorShade::S6 => 0,
            ColorShade::S7 => -43,
            ColorShade::S8 => -59,
            ColorShade::S9 => -75,
        },
    }
}

/// Where a ramp's far steps mix towards: a neutral ramp to the theme's `ink`, a chromatic
/// one to pure black or white opposite the page, or a weak ink would cap accents below 4.5:1.
const fn far_end(ramp: ShadeRamp, ends: Ends) -> HexColor {
    match ramp {
        ShadeRamp::Neutral => ends.ink,
        ShadeRamp::Chromatic => ends.surface.contrast(),
    }
}

const fn mix_channel(base: u8, other: u8, weight: u8) -> u8 {
    (((base as u16 * (100 - weight) as u16) + (other as u16 * weight as u16)) / 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_color_happy_path() {
        const COLOR: HexColor = HexColor::new(0x228BE6);

        assert_eq!(COLOR.r(), 0x22);
        assert_eq!(COLOR.g(), 0x8B);
        assert_eq!(COLOR.b(), 0xE6);
        assert_eq!(COLOR.rgb(), 0x228BE6);
    }

    #[test]
    fn hex_color_shade_and_contrast() {
        const COLOR: HexColor = HexColor::new(0x228BE6);
        const SHADE: HexColor = COLOR.shade(ColorShade::S1, ShadeRamp::Chromatic, Ends::ABSOLUTE);
        const CONTRAST: HexColor = COLOR.contrast();

        assert_eq!(SHADE.rgb(), 0xD2_E7_FA);
        assert_eq!(CONTRAST.rgb(), 0xFF_FF_FF);
    }

    /// Ratios measured in Chromium (todo 239).
    #[test]
    fn contrast_ratio_matches_what_the_browser_measured() {
        const WHITE: HexColor = HexColor::new(0xFF_FF_FF);

        let ratio = |rgb: u32| HexColor::new(rgb).contrast_ratio(WHITE);

        assert!((ratio(0x22_8B_E6) - 3.56).abs() < 0.01, "blue.6");
        assert!((ratio(0x1C_74_C1) - 4.86).abs() < 0.01, "blue.8");
        assert!((ratio(0x86_8E_96) - 3.32).abs() < 0.01, "muted.6");
        assert!((ratio(0x4C_50_55) - 8.12).abs() < 0.01, "muted.7");
    }

    /// `muted.6`, every control outline's shade, is 3:1 on page and `Paper`; `muted.5`
    /// is not (WCAG 1.4.11, todo 490).
    #[test]
    fn muted_6_is_the_first_boundary_shade_at_3_to_1() {
        // Measured against the library themes: exempt from the layer rule.
        // archunit: ignore theme
        use crate::theme::Theme;

        for theme in [Theme::DEFAULT, Theme::DARK] {
            let ends = Ends {
                surface: theme.surface,
                ink: theme.ink,
            };
            let paper = HexColor::parse(theme.paper.background).expect("a hex paper");
            let muted = |shade| theme.muted.shade(shade, ShadeRamp::Neutral, ends);
            for background in [theme.surface, paper] {
                let ratio = muted(ColorShade::S6).contrast_ratio(background);
                assert!(ratio >= 3.0, "muted.6 on {background}: {ratio:.2}");
            }
            let ratio = muted(ColorShade::S5).contrast_ratio(theme.surface);
            assert!(ratio < 3.0, "muted.5 on {}: {ratio:.2}", theme.surface);
        }
    }

    /// `blue.6` + white is 3.56:1, so the fill darkens, only as far as the pair needs.
    #[test]
    fn the_two_roles_move_only_as_far_as_they_must() {
        const BLUE: HexColor = HexColor::new(0x22_8B_E6);
        const WHITE: HexColor = HexColor::new(0xFF_FF_FF);

        let text = BLUE.text_base(ShadeRamp::Chromatic, Ends::ABSOLUTE, &[]);
        let fill = BLUE.fill_base(ShadeRamp::Chromatic, Ends::ABSOLUTE);
        assert!(text.contrast_ratio(WHITE) >= TEXT_CONTRAST);
        assert!(fill.contrast_ratio(WHITE) >= TEXT_CONTRAST);
        // One percent less of the mix would fail: nothing overshoots.
        for weight in 0..(0..=25)
            .find(|&w| BLUE.mix(HexColor::new(0), w) == text)
            .expect("a mix")
        {
            assert!(BLUE.mix(HexColor::new(0), weight).contrast_ratio(WHITE) < TEXT_CONTRAST);
        }
        // Closer to the brand than the whole-step walk's `blue.8` was.
        let brand_distance = |c: HexColor| c.contrast_ratio(BLUE);
        assert!(brand_distance(text) < brand_distance(HexColor::new(0x1C_74_C1)));

        // `fill_base` asks `contrast()`: `readable_contrast()` would label blue in black (5.90:1).
        assert_eq!(BLUE.contrast(), WHITE);
        assert_eq!(
            BLUE.readable_contrast(Ends::ABSOLUTE),
            HexColor::new(0x00_00_00)
        );
        assert_eq!(fill.readable_contrast(Ends::ABSOLUTE), WHITE);

        // `green.6` already carries black, so its fill stays where it is.
        const GREEN: HexColor = HexColor::new(0x40_C0_57);
        assert_eq!(GREEN.fill_base(ShadeRamp::Chromatic, Ends::ABSOLUTE), GREEN);
    }

    /// `Theme::DARK`'s blue: 4.88:1 on the page, 4.24:1 on dialogs' `Paper` (todo 314).
    #[test]
    fn the_text_role_reads_on_the_card_as_well_as_the_page() {
        const BLUE: HexColor = HexColor::new(0x22_8B_E6);
        const PAGE: HexColor = HexColor::new(0x1A_1B_1E);
        const CARD: HexColor = HexColor::new(0x25_26_2B);
        let ends = Ends {
            surface: PAGE,
            ink: HexColor::new(0xE9_EC_EF),
        };

        assert_eq!(BLUE.text_base(ShadeRamp::Chromatic, ends, &[]), BLUE);
        let text = BLUE.text_base(ShadeRamp::Chromatic, ends, &[CARD]);
        assert!(text.contrast_ratio(CARD) >= TEXT_CONTRAST);
        assert!(text.contrast_ratio(PAGE) >= TEXT_CONTRAST);
        // The smallest mix that does it, lighter on an inked page.
        let far = HexColor::new(0xFF_FF_FF);
        let weight = (0..=25).find(|&w| BLUE.mix(far, w) == text).expect("a mix");
        assert!(weight > 0);
        assert!(BLUE.mix(far, weight - 1).contrast_ratio(CARD) < TEXT_CONTRAST);
    }

    #[test]
    fn hex_color_parse_recognizes_hex_and_opaque_rgb() {
        assert_eq!(HexColor::parse("#fff"), Some(HexColor::new(0xFF_FF_FF)));
        assert_eq!(HexColor::parse("#228BE6"), Some(HexColor::new(0x22_8B_E6)));
        assert_eq!(
            HexColor::parse("rgb(34, 139, 230)"),
            Some(HexColor::new(0x22_8B_E6))
        );
        assert_eq!(
            HexColor::parse("rgba(34, 139, 230, 1)"),
            Some(HexColor::new(0x22_8B_E6))
        );
    }

    #[test]
    fn hex_color_parse_rejects_translucent_rgba_and_unknown_formats() {
        assert_eq!(HexColor::parse("rgba(255, 255, 255, 0.15)"), None);
        assert_eq!(HexColor::parse("hsl(0, 0%, 100%)"), None);
        assert_eq!(HexColor::parse("green"), None);
    }
}
