use std::fmt::{self, Display};

use super::{ColorShade, ShadeRamp};

/// WCAG 1.4.3 for text below 18.66px bold / 24px. Every colour this library
/// resolves for a text or fill role is held to it, because a component cannot
/// know how big the text on it will be.
pub(crate) const TEXT_CONTRAST: f32 = 4.5;

/// Which way [`HexColor::first_shade_from`] walks a ramp looking for the first
/// step that reads. A ramp only ever gains contrast in one direction, and
/// which one depends on the surface it is measured against (todo 69).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShadeWalk {
    Darker,
    Lighter,
}

impl ShadeWalk {
    /// Away from `surface`: darker on a light one, lighter on a dark one.
    /// `contrast()` is the same brightness split the rest of the role system
    /// sorts by, so the two never disagree about which end a surface is at.
    fn away_from(surface: HexColor) -> Self {
        if surface.contrast().rgb() == 0x00_00_00 {
            Self::Darker
        } else {
            Self::Lighter
        }
    }

    const fn step(self, shade: ColorShade) -> ColorShade {
        match self {
            Self::Darker => shade.darker(),
            Self::Lighter => shade.lighter(),
        }
    }

    const fn end(self) -> ColorShade {
        match self {
            Self::Darker => ColorShade::S9,
            Self::Lighter => ColorShade::S1,
        }
    }
}

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

    /// `#rgb`/`#rrggbb`, `rgb()`, and fully-opaque `rgba()`. A translucent
    /// `rgba()` is `None` on purpose - its visual color depends on whatever
    /// shows through. Named colors, `hsl()` and vars are `None` too.
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

    /// Fitted to Mantine's palettes, so a Mantine shade-6 base reproduces
    /// its ramp.
    pub(crate) const fn shade(self, shade: ColorShade, ramp: ShadeRamp) -> Self {
        let percent = match ramp {
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
        };

        if percent >= 0 {
            self.mix(HexColor::new(0xFF_FF_FF), percent as u8)
        } else {
            self.mix(HexColor::new(0x00_00_00), (-percent) as u8)
        }
    }

    /// Mantine's `autoContrast`: black or white, picked by perceived
    /// brightness. It says which foreground *belongs* on this fill, not
    /// whether that foreground passes - ask [`contrast_ratio`] for that.
    ///
    /// [`contrast_ratio`]: Self::contrast_ratio
    pub(crate) const fn contrast(self) -> Self {
        if self.luminance() >= 140 {
            HexColor::new(0x00_00_00)
        } else {
            HexColor::new(0xFF_FF_FF)
        }
    }

    /// Which `color-scheme` a surface of this colour is, so native controls
    /// and the canvas are drawn on the same end of the greyscale as the rest
    /// of the theme.
    pub(crate) const fn color_scheme(self) -> &'static str {
        if self.contrast().rgb() == 0xFF_FF_FF {
            "dark"
        } else {
            "light"
        }
    }

    /// WCAG 2.x relative luminance. Not [`luminance`](Self::luminance): that
    /// one is the cheap integer approximation `contrast()` sorts by, and it
    /// is nowhere near the curve 1.4.3 is defined on.
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

    /// The foreground for this colour: [`contrast`](Self::contrast)'s pick,
    /// unless it fails [`TEXT_CONTRAST`] and the other one does better.
    ///
    /// The fallback is for the steps the fill ramp is *not* re-based on -
    /// the mid steps, where the brightness threshold and the WCAG curve
    /// disagree. It never overrides the pick on a step that passes, which is
    /// why a `blue` fill still gets white and is darkened instead of being
    /// labelled in black.
    pub(crate) fn readable_contrast(self) -> Self {
        let picked = self.contrast();
        if self.contrast_ratio(picked) >= TEXT_CONTRAST {
            return picked;
        }

        let other = if picked.rgb() == 0x00_00_00 {
            HexColor::new(0xFF_FF_FF)
        } else {
            HexColor::new(0x00_00_00)
        };

        if self.contrast_ratio(other) > self.contrast_ratio(picked) {
            other
        } else {
            picked
        }
    }

    /// The step a *text* use of `shade` resolves to: the first step from it
    /// **away from `surface`** whose colour passes [`TEXT_CONTRAST`] against
    /// it. On a light surface the ramp gains contrast as it darkens, on a
    /// dark one as it lightens, so the walk follows the surface rather than
    /// always heading for `S9`. The end of that walk when the whole ramp fails,
    /// because there is nothing further to offer and a wrong-looking colour
    /// beats no colour.
    ///
    /// `self` is the base (shade 6) of the ramp, as in [`shade`](Self::shade).
    pub(crate) fn text_shade(
        self,
        ramp: ShadeRamp,
        shade: ColorShade,
        surface: HexColor,
    ) -> ColorShade {
        self.first_shade_from(shade, ShadeWalk::away_from(surface), |step| {
            self.shade(step, ramp).contrast_ratio(surface) >= TEXT_CONTRAST
        })
    }

    /// The step a *fill* use of `shade` resolves to: the first step at or
    /// below it on which the foreground [`contrast`](Self::contrast) picks
    /// passes [`TEXT_CONTRAST`]. Mantine's `autoContrast` stops at picking the
    /// foreground and leaves a fill like `blue.6` - where white is picked and
    /// only reaches 3.56:1 - alone; this walks the ramp until the pair works.
    /// No `surface` argument on purpose: a fill carries its own foreground,
    /// so the pair reads the same on any surface and the walk is always
    /// downwards.
    pub(crate) fn fill_shade(self, ramp: ShadeRamp, shade: ColorShade) -> ColorShade {
        self.first_shade_from(shade, ShadeWalk::Darker, |step| {
            let fill = self.shade(step, ramp);
            fill.contrast_ratio(fill.contrast()) >= TEXT_CONTRAST
        })
    }

    fn first_shade_from(
        self,
        shade: ColorShade,
        walk: ShadeWalk,
        passes: impl Fn(ColorShade) -> bool,
    ) -> ColorShade {
        let mut step = shade;
        loop {
            if passes(step) || step == walk.end() {
                return step;
            }
            step = walk.step(step);
        }
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
        const SHADE: HexColor = COLOR.shade(ColorShade::S1, ShadeRamp::Chromatic);
        const CONTRAST: HexColor = COLOR.contrast();

        assert_eq!(SHADE.rgb(), 0xD2_E7_FA);
        assert_eq!(CONTRAST.rgb(), 0xFF_FF_FF);
    }

    /// The two ratios review 4 measured in Chromium (todo 239), so a change
    /// to the maths is caught against a browser's own numbers.
    #[test]
    fn contrast_ratio_matches_what_the_browser_measured() {
        const WHITE: HexColor = HexColor::new(0xFF_FF_FF);

        let ratio = |rgb: u32| HexColor::new(rgb).contrast_ratio(WHITE);

        assert!((ratio(0x22_8B_E6) - 3.56).abs() < 0.01, "blue.6");
        assert!((ratio(0x1C_74_C1) - 4.86).abs() < 0.01, "blue.8");
        assert!((ratio(0x86_8E_96) - 3.32).abs() < 0.01, "grey.6");
        assert!((ratio(0x4C_50_55) - 8.12).abs() < 0.01, "grey.7");
    }

    /// `blue.6` is the case Mantine's `autoContrast` alone cannot fix: white
    /// is the right foreground for it and only reaches 3.56:1, so the fill
    /// walks down the ramp instead of relabelling itself in black.
    #[test]
    fn the_two_roles_walk_the_ramp_until_they_read() {
        const BLUE: HexColor = HexColor::new(0x22_8B_E6);
        const WHITE: HexColor = HexColor::new(0xFF_FF_FF);

        assert_eq!(
            BLUE.text_shade(ShadeRamp::Chromatic, ColorShade::S6, WHITE),
            ColorShade::S8
        );
        assert_eq!(
            BLUE.fill_shade(ShadeRamp::Chromatic, ColorShade::S6),
            ColorShade::S8
        );
        // Why `fill_shade` asks `contrast()` and not `readable_contrast()`:
        // black *does* clear 4.5:1 on `blue.6` (5.90:1), so the readable pick
        // would keep the fill and label the button in black. White is the
        // foreground a blue fill wants; the fill moves instead.
        assert_eq!(BLUE.contrast(), WHITE);
        assert_eq!(BLUE.readable_contrast(), HexColor::new(0x00_00_00));
        assert_eq!(HexColor::new(0x1C_74_C1).readable_contrast(), WHITE);

        // `green.6` already carries black, so its fill stays where it is.
        const GREEN: HexColor = HexColor::new(0x40_C0_57);
        assert_eq!(
            GREEN.fill_shade(ShadeRamp::Chromatic, ColorShade::S6),
            ColorShade::S6
        );
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
