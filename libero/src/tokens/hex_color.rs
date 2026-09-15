use std::fmt::{self, Display};

use super::{ColorShade, ShadeRamp};

/// WCAG 1.4.3 for text below 18.66px bold / 24px. Every colour this library
/// resolves for a text or fill role is held to it, because a component cannot
/// know how big the text on it will be.
pub(crate) const TEXT_CONTRAST: f32 = 4.5;

/// The two ends of a theme's page: the `surface` it is painted on and the
/// `ink` text is set in. Carried together because a neutral ramp is mixed
/// between them and every role derivation is measured against one of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Ends {
    pub(crate) surface: HexColor,
    pub(crate) ink: HexColor,
}

impl Ends {
    /// Which of the two ends `pick` - always pure black or pure white,
    /// because that is all [`HexColor::contrast`] returns - is painted as.
    ///
    /// Not a black/white lookup: on a dark theme the ink is the *light* end,
    /// so a fill that wants a white label is painted in the ink, and the ink
    /// is not pure white. Everything that measures a foreground measures the
    /// colour this returns, or it flatters the pair.
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

    /// The light theme's own ends. Nothing in the library mixes against
    /// absolute white and black any more - every ramp uses the theme's ends -
    /// so this is the tests' stand-in for "the default theme".
    #[cfg(test)]
    pub(crate) const ABSOLUTE: Ends = Ends {
        surface: HexColor::new(0xFF_FF_FF),
        ink: HexColor::new(0x00_00_00),
    };
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
    ///
    /// **A neutral ramp mixes between the theme's own ends, a chromatic one
    /// between absolute white and black.** The neutral ramp is not a
    /// lightness scale, it is distance from the page: S1 is the step that
    /// barely separates from the surface - a hover background - and S9 the
    /// one furthest from it, up against the ink. Mixing it towards white
    /// would put every subtle background at the wrong end of a dark theme,
    /// and mixing it towards black would overshoot a surface that is not
    /// itself black. On a light theme `ends` is white and black, so this is
    /// exactly what it emitted before (todo 69 phase 3).
    pub(crate) const fn shade(self, shade: ColorShade, ramp: ShadeRamp, ends: Ends) -> Self {
        let percent = mix_percent(ramp, shade);
        // The near end is the page itself; the far end is `far_end`'s.
        let far = far_end(ramp, ends);

        if percent >= 0 {
            self.mix(ends.surface, percent as u8)
        } else {
            self.mix(far, (-percent) as u8)
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

    /// The colour a *text* use of this base resolves to: the base itself
    /// when it passes [`TEXT_CONTRAST`] on the surface, and otherwise the
    /// **smallest** mix towards the ramp's far end that does - the end
    /// furthest from the page, so darker on a paper and lighter on an inked
    /// one. The far end itself when nothing passes, because there is nothing
    /// further to offer and a wrong-looking colour beats no colour.
    ///
    /// The smallest mix, not the next ramp step: a step is up to 9% of a mix
    /// away from the last, and jumping a whole one moved a palette's accent
    /// visibly further from the colour its author picked than contrast asked
    /// for. The ramps derived from this base are still ordinary ramps.
    ///
    /// `self` is the base (shade 6) of the ramp, as in [`shade`](Self::shade).
    /// `cards` are the theme's other surfaces text lands on (a dark `Paper`
    /// one step off the page, todo 314); it must read on those too.
    pub(crate) fn text_base(self, ramp: ShadeRamp, ends: Ends, cards: &[Self]) -> Self {
        self.first_mix(ramp, ends, |text| {
            text.contrast_ratio(ends.surface) >= TEXT_CONTRAST
                && cards
                    .iter()
                    .all(|&card| text.contrast_ratio(card) >= TEXT_CONTRAST)
        })
    }

    /// The colour a *fill* use of this base resolves to: the smallest mix
    /// towards the far end on which the foreground [`contrast`](Self::contrast)
    /// picks passes [`TEXT_CONTRAST`]. Mantine's `autoContrast` stops at
    /// picking the foreground and leaves a fill like `blue.6` - where white is
    /// picked and only reaches 3.56:1 - alone; this moves the fill until the
    /// pair works.
    ///
    /// It moves away from the surface for the same reason the text one does.
    /// On a paper page the fill darkens into white labels; on an inked one it
    /// lightens into dark ones, which is also what keeps a filled control
    /// visibly separate from the page it sits on. `info` is the case that
    /// proves it: darkening on the dark theme runs out at `#0F7F8F`, whose
    /// label lands at 3.98:1, and leaves a button 2:1 off its own page.
    ///
    /// The foreground measured is the one the theme paints - `ends.ink` or
    /// `ends.surface` - not the pure black or white that picked the side.
    pub(crate) fn fill_base(self, ramp: ShadeRamp, ends: Ends) -> Self {
        self.first_mix(ramp, ends, |fill| {
            fill.contrast_ratio(ends.foreground(fill.contrast())) >= TEXT_CONTRAST
        })
    }

    /// [`text_base`](Self::text_base) for text drawn on other surfaces than
    /// the page: the smallest mix towards black or white, opposite the page,
    /// that reads at `floor` on every background. Any colour, not only a base.
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

    /// Walks the mix from this base towards the ramp's far end one percent at
    /// a time, as far as [`ColorShade::S9`] goes, and stops at the first
    /// colour that `passes`.
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
/// the surface when positive, towards [`far_end`] when negative. Fitted to
/// Mantine's palettes.
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

/// Where a ramp's dark-side steps mix towards. The near end is the page
/// itself, in both ramps: a low step is a tint of the surface it will be
/// drawn on. The far end differs. A neutral ramp ends at the theme's own
/// `ink`, because its far steps *are* text. A chromatic one ends at pure black
/// or pure white, whichever is opposite the page: its far steps exist to be
/// legible, and a palette whose ink is only 6:1 on its own page would
/// otherwise cap every accent's text role below 4.5:1.
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

    /// The two ratios review 4 measured in Chromium (todo 239), so a change
    /// to the maths is caught against a browser's own numbers.
    #[test]
    fn contrast_ratio_matches_what_the_browser_measured() {
        const WHITE: HexColor = HexColor::new(0xFF_FF_FF);

        let ratio = |rgb: u32| HexColor::new(rgb).contrast_ratio(WHITE);

        assert!((ratio(0x22_8B_E6) - 3.56).abs() < 0.01, "blue.6");
        assert!((ratio(0x1C_74_C1) - 4.86).abs() < 0.01, "blue.8");
        assert!((ratio(0x86_8E_96) - 3.32).abs() < 0.01, "muted.6");
        assert!((ratio(0x4C_50_55) - 8.12).abs() < 0.01, "muted.7");
    }

    /// `muted.6` is the boundary shade of every control outline, off track and
    /// pending ring: 3:1 on the page and on `Paper` in both library themes,
    /// and `muted.5` below it is not (WCAG 1.4.11, todo 490).
    #[test]
    fn muted_6_is_the_first_boundary_shade_at_3_to_1() {
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

    /// `blue.6` is the case Mantine's `autoContrast` alone cannot fix: white
    /// is the right foreground for it and only reaches 3.56:1, so the fill
    /// darkens instead of relabelling itself in black - but only as far as
    /// the pair needs, not a whole ramp step.
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

        // Why `fill_base` asks `contrast()` and not `readable_contrast()`:
        // black *does* clear 4.5:1 on `blue.6` (5.90:1), so the readable pick
        // would keep the fill and label the button in black. White is the
        // foreground a blue fill wants; the fill moves instead.
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

    /// `Theme::DARK`'s blue reads 4.88:1 on its page but 4.24:1 on the
    /// `Paper` one step off it, where every dialog puts its buttons (todo 314).
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
