use std::{
    f64::consts::PI,
    fmt::{self, Display},
    str::FromStr,
};

use crate::theme::{ColorFormat, HexColor};

/// A color as the picker holds it: HSVA, so a grey keeps its hue.
///
/// Parses from any CSS hex, rgb or hsl form (`"#228be6".parse()`); writes back
/// with [`to_hex`](Self::to_hex), [`to_rgba`](Self::to_rgba) and the rest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorCode {
    /// Degrees, `0.0..360.0`.
    hue: f64,
    /// `0.0..=1.0`.
    saturation: f64,
    /// `0.0..=1.0`.
    value: f64,
    /// `0.0..=1.0`.
    alpha: f64,
}

/// A string no CSS color form matches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseColorError;

impl Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not a hex, rgb or hsl color")
    }
}

impl std::error::Error for ParseColorError {}

impl Default for ColorCode {
    /// Opaque black.
    fn default() -> Self {
        Self::hsva(0.0, 0.0, 0.0, 1.0)
    }
}

impl ColorCode {
    /// Hue in degrees; saturation, value and alpha as `0.0..=1.0`. Out of
    /// range inputs are wrapped (hue) or clamped (the rest); a non-finite hue is 0.
    pub fn hsva(hue: f64, saturation: f64, value: f64, alpha: f64) -> Self {
        Self {
            hue: if hue.is_finite() {
                hue.rem_euclid(360.0)
            } else {
                0.0
            },
            saturation: unit(saturation),
            value: unit(value),
            alpha: unit(alpha),
        }
    }

    /// Channels as `0-255`, alpha as `0.0..=1.0`.
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: f64) -> Self {
        let (r, g, b) = (
            red as f64 / 255.0,
            green as f64 / 255.0,
            blue as f64 / 255.0,
        );
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let hue = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / delta).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        let saturation = if max == 0.0 { 0.0 } else { delta / max };

        Self::hsva(hue, saturation, max, alpha)
    }

    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// Hue in degrees; saturation, lightness and alpha as `0.0..=1.0`.
    pub fn hsla(hue: f64, saturation: f64, lightness: f64, alpha: f64) -> Self {
        let (saturation, lightness) = (unit(saturation), unit(lightness));
        let value = lightness + saturation * lightness.min(1.0 - lightness);
        let saturation = if value == 0.0 {
            0.0
        } else {
            2.0 * (1.0 - lightness / value)
        };
        Self::hsva(hue, saturation, value, alpha)
    }

    pub fn hsl(hue: f64, saturation: f64, lightness: f64) -> Self {
        Self::hsla(hue, saturation, lightness, 1.0)
    }

    /// `0xRRGGBB`, opaque.
    pub fn hex(rgb: u32) -> Self {
        Self::rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }

    /// Degrees, `0.0..360.0`.
    pub fn hue(self) -> f64 {
        self.hue
    }

    pub fn saturation(self) -> f64 {
        self.saturation
    }

    pub fn value(self) -> f64 {
        self.value
    }

    pub fn alpha(self) -> f64 {
        self.alpha
    }

    /// The same color at another hue.
    pub fn with_hue(self, hue: f64) -> Self {
        Self::hsva(hue, self.saturation, self.value, self.alpha)
    }

    /// The same hue and alpha at another point of the saturation panel.
    pub fn with_saturation_value(self, saturation: f64, value: f64) -> Self {
        Self::hsva(self.hue, saturation, value, self.alpha)
    }

    pub fn with_alpha(self, alpha: f64) -> Self {
        Self::hsva(self.hue, self.saturation, self.value, alpha)
    }

    /// Fully opaque, which is what an alpha slider's gradient runs towards.
    pub fn opaque(self) -> Self {
        self.with_alpha(1.0)
    }

    /// Channels as `0-255`, alpha as `0.0..=1.0`.
    pub fn to_rgba_channels(self) -> (u8, u8, u8, f64) {
        let chroma = self.value * self.saturation;
        let sector = self.hue / 60.0;
        let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
        let (r, g, b) = match sector as u32 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let m = self.value - chroma;
        let channel = |c: f64| ((c + m) * 255.0).round() as u8;
        (channel(r), channel(g), channel(b), self.alpha)
    }

    /// Hue in degrees; saturation, lightness and alpha as `0.0..=1.0`.
    pub fn to_hsla_channels(self) -> (f64, f64, f64, f64) {
        let lightness = self.value * (1.0 - self.saturation / 2.0);
        let saturation = if lightness == 0.0 || lightness == 1.0 {
            0.0
        } else {
            (self.value - lightness) / lightness.min(1.0 - lightness)
        };
        (self.hue, saturation, lightness, self.alpha)
    }

    /// `#228be6`. Alpha is dropped.
    pub fn to_hex(self) -> String {
        let (r, g, b, _) = self.to_rgba_channels();
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// `#228be680`.
    pub fn to_hexa(self) -> String {
        let (r, g, b, a) = self.to_rgba_channels();
        format!("#{r:02x}{g:02x}{b:02x}{:02x}", (a * 255.0).round() as u8)
    }

    /// `rgb(34, 139, 230)`. Alpha is dropped.
    pub fn to_rgb(self) -> String {
        let (r, g, b, _) = self.to_rgba_channels();
        format!("rgb({r}, {g}, {b})")
    }

    /// `rgba(34, 139, 230, 0.5)`.
    pub fn to_rgba(self) -> String {
        let (r, g, b, a) = self.to_rgba_channels();
        format!("rgba({r}, {g}, {b}, {})", round_alpha(a))
    }

    /// `hsl(208, 80%, 52%)`, rounded to whole degrees and percent. Alpha is
    /// dropped.
    pub fn to_hsl(self) -> String {
        let (h, s, l, _) = self.to_hsla_channels();
        format!(
            "hsl({}, {}%, {}%)",
            h.round() as u16 % 360,
            (s * 100.0).round(),
            (l * 100.0).round()
        )
    }

    /// `hsla(208, 80%, 52%, 0.5)`.
    pub fn to_hsla(self) -> String {
        let (h, s, l, a) = self.to_hsla_channels();
        format!(
            "hsla({}, {}%, {}%, {})",
            h.round() as u16 % 360,
            (s * 100.0).round(),
            (l * 100.0).round(),
            round_alpha(a)
        )
    }

    /// The text of one CSS form, picked at runtime.
    pub fn to_format(self, format: ColorFormat) -> String {
        match format {
            ColorFormat::Hex => self.to_hex(),
            ColorFormat::Hexa => self.to_hexa(),
            ColorFormat::Rgb => self.to_rgb(),
            ColorFormat::Rgba => self.to_rgba(),
            ColorFormat::Hsl => self.to_hsl(),
            ColorFormat::Hsla => self.to_hsla(),
        }
    }
}

impl Display for ColorCode {
    /// `#rrggbb` when opaque, `#rrggbbaa` otherwise - what a `style` accepts.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.alpha >= 1.0 {
            true => f.write_str(&self.to_hex()),
            false => f.write_str(&self.to_hexa()),
        }
    }
}

impl From<HexColor> for ColorCode {
    fn from(color: HexColor) -> Self {
        Self::rgb(color.r(), color.g(), color.b())
    }
}

impl FromStr for ColorCode {
    type Err = ParseColorError;

    /// Hex with 3, 4, 6 or 8 digits (`#` optional), `rgb[a]()` and `hsl[a]()`,
    /// comma or space separated, rgb channels and alpha as a number or percentage,
    /// hue in any CSS angle unit.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim().to_ascii_lowercase();

        if let Some(inner) = function_args(&text, "rgba").or_else(|| function_args(&text, "rgb")) {
            let ([r, g, b], a) = arguments(inner)?;
            return Ok(Self::rgba(
                byte(r)?,
                byte(g)?,
                byte(b)?,
                a.map_or(Ok(1.0), alpha)?,
            ));
        }
        if let Some(inner) = function_args(&text, "hsla").or_else(|| function_args(&text, "hsl")) {
            let ([h, s, l], a) = arguments(inner)?;
            return Ok(Self::hsla(
                hue(h)?,
                percent(s)?,
                percent(l)?,
                a.map_or(Ok(1.0), alpha)?,
            ));
        }

        parse_hex(text.strip_prefix('#').unwrap_or(&text))
    }
}

fn unit(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}

/// Two decimals, with no trailing zeros - `0.5`, not `0.50`.
pub(super) fn round_alpha(alpha: f64) -> f64 {
    (alpha * 100.0).round() / 100.0
}

fn function_args<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    text.strip_prefix(name)?
        .trim_start()
        .strip_prefix('(')?
        .strip_suffix(')')
}

/// Three channels and an optional alpha, split on commas, spaces or a `/`.
fn arguments(inner: &str) -> Result<([&str; 3], Option<&str>), ParseColorError> {
    let mut parts = inner
        .split([',', ' ', '/'])
        .map(str::trim)
        .filter(|part| !part.is_empty());
    let channels = [
        parts.next().ok_or(ParseColorError)?,
        parts.next().ok_or(ParseColorError)?,
        parts.next().ok_or(ParseColorError)?,
    ];
    let alpha = parts.next();
    match parts.next() {
        Some(_) => Err(ParseColorError),
        None => Ok((channels, alpha)),
    }
}

fn number(text: &str) -> Result<f64, ParseColorError> {
    text.parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or(ParseColorError)
}

/// `255` or `100%`.
fn byte(text: &str) -> Result<u8, ParseColorError> {
    let value = match text.strip_suffix('%') {
        Some(percent) => number(percent)? / 100.0 * 255.0,
        None => number(text)?,
    };
    match (0.0..=255.0).contains(&value) {
        true => Ok(value.round() as u8),
        false => Err(ParseColorError),
    }
}

/// A CSS hue in degrees: bare, `deg`, `grad`, `rad` or `turn`.
fn hue(text: &str) -> Result<f64, ParseColorError> {
    let units = [
        ("deg", 1.0),
        ("grad", 0.9),
        ("rad", 180.0 / PI),
        ("turn", 360.0),
    ];
    match units
        .iter()
        .find_map(|(unit, scale)| Some((text.strip_suffix(unit)?, scale)))
    {
        Some((value, scale)) => Ok(number(value)? * scale),
        None => number(text),
    }
}

/// `80%` or `0.8`, as `0.0..=1.0`.
fn percent(text: &str) -> Result<f64, ParseColorError> {
    let value = match text.strip_suffix('%') {
        Some(percent) => number(percent)? / 100.0,
        None => number(text)? / 100.0,
    };
    match (0.0..=1.0).contains(&value) {
        true => Ok(value),
        false => Err(ParseColorError),
    }
}

/// `0.5` or `50%`.
fn alpha(text: &str) -> Result<f64, ParseColorError> {
    let value = match text.strip_suffix('%') {
        Some(percent) => number(percent)? / 100.0,
        None => number(text)?,
    };
    match (0.0..=1.0).contains(&value) {
        true => Ok(value),
        false => Err(ParseColorError),
    }
}

fn parse_hex(hex: &str) -> Result<ColorCode, ParseColorError> {
    if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ParseColorError);
    }
    let digit = |index: usize| u8::from_str_radix(&hex[index..=index], 16).unwrap_or(0);
    let pair = |index: usize| u8::from_str_radix(&hex[index..index + 2], 16).unwrap_or(0);

    let (r, g, b, a) = match hex.len() {
        3 | 4 => (
            digit(0) * 17,
            digit(1) * 17,
            digit(2) * 17,
            (hex.len() == 4).then(|| digit(3) * 17),
        ),
        6 | 8 => (pair(0), pair(2), pair(4), (hex.len() == 8).then(|| pair(6))),
        _ => return Err(ParseColorError),
    };
    Ok(ColorCode::rgba(
        r,
        g,
        b,
        a.map_or(1.0, |a| a as f64 / 255.0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> ColorCode {
        text.parse()
            .unwrap_or_else(|_| panic!("{text} should parse"))
    }

    #[test]
    fn every_css_form_parses_to_the_same_color() {
        let expected = ColorCode::rgb(34, 139, 230).to_rgba_channels();
        for text in [
            "#228be6",
            "#228BE6",
            "228be6",
            "#228be6ff",
            "rgb(34, 139, 230)",
            "rgba(34, 139, 230, 1)",
            "rgb(34 139 230)",
            "rgb(34 139 230 / 100%)",
            "  RGB(34,139,230)  ",
        ] {
            assert_eq!(parse(text).to_rgba_channels(), expected, "{text}");
        }
        assert_eq!(parse("#fff").to_hex(), "#ffffff");
        assert_eq!(parse("#0008").to_hexa(), "#00000088");
    }

    #[test]
    fn hsl_parses_with_or_without_percent_signs() {
        assert_eq!(parse("hsl(208, 80%, 52%)").to_hsl(), "hsl(208, 80%, 52%)");
        assert_eq!(
            parse("hsla(208deg 80% 52% / 0.5)").to_hsla(),
            "hsla(208, 80%, 52%, 0.5)"
        );
        assert_eq!(parse("hsl(120, 100, 50)").to_hex(), "#00ff00");
    }

    /// Todo 2295: percent rgb channels and every CSS hue unit.
    #[test]
    fn percent_channels_and_hue_units_parse() {
        assert_eq!(parse("rgb(100% 0% 0%)").to_hex(), "#ff0000");
        assert_eq!(parse("rgb(50%, 50%, 50%, 50%)").to_hexa(), "#80808080");
        assert_eq!(parse("rgb(50% 50% 50% / 50%)").to_hexa(), "#80808080");
        for text in [
            "hsl(120deg 100% 50%)",
            "hsl(0.3333turn 100% 50%)",
            "hsl(133.33grad 100% 50%)",
            "hsl(2.0944rad 100% 50%)",
        ] {
            assert_eq!(parse(text).to_hex(), "#00ff00", "{text}");
        }
        assert_eq!(
            "rgb(101%, 0%, 0%)".parse::<ColorCode>(),
            Err(ParseColorError)
        );
    }

    #[test]
    fn nonsense_is_an_error_not_a_panic() {
        for text in [
            "",
            "#",
            "#12",
            "#12345",
            "#ggg",
            "green",
            "rgb(1, 2)",
            "rgb(1, 2, 3, 4, 5)",
            "rgb(300, 0, 0)",
            "rgba(0, 0, 0, 2)",
            "hsl(0, 120%, 50%)",
            "rgb(a, b, c)",
            "rgb(nan, 0, 0)",
            "#€€€",
        ] {
            assert_eq!(text.parse::<ColorCode>(), Err(ParseColorError), "{text}");
        }
    }

    #[test]
    fn every_eight_bit_color_survives_the_hsv_round_trip() {
        for r in (0..=255).step_by(15) {
            for g in (0..=255).step_by(15) {
                for b in (0..=255).step_by(15) {
                    let rgb = (r as u8, g as u8, b as u8);
                    let (r2, g2, b2, _) = ColorCode::rgb(rgb.0, rgb.1, rgb.2).to_rgba_channels();
                    assert_eq!((r2, g2, b2), rgb);
                }
            }
        }
        // Neighbours of the sector edges, where rounding would slip first.
        for rgb in [
            (1, 0, 0),
            (254, 255, 0),
            (0, 1, 2),
            (128, 127, 128),
            (255, 254, 253),
        ] {
            let (r, g, b, _) = ColorCode::rgb(rgb.0, rgb.1, rgb.2).to_rgba_channels();
            assert_eq!((r, g, b), rgb);
        }
    }

    #[test]
    fn hue_survives_grey_and_black() {
        let blue = ColorCode::hsva(210.0, 1.0, 1.0, 1.0);
        assert_eq!(blue.with_saturation_value(0.0, 0.0).hue(), 210.0);
        assert_eq!(
            blue.with_saturation_value(0.0, 0.5).to_hsla_channels().0,
            210.0
        );
    }

    #[test]
    fn formats_print_what_css_expects() {
        let color = ColorCode::rgba(34, 139, 230, 0.5);
        assert_eq!(color.to_hex(), "#228be6");
        assert_eq!(color.to_hexa(), "#228be680");
        assert_eq!(color.to_rgb(), "rgb(34, 139, 230)");
        assert_eq!(color.to_rgba(), "rgba(34, 139, 230, 0.5)");
        assert_eq!(color.to_hsl(), "hsl(208, 80%, 52%)");
        assert_eq!(color.to_hsla(), "hsla(208, 80%, 52%, 0.5)");
        assert_eq!(color.to_format(ColorFormat::Rgba), color.to_rgba());
        assert_eq!(color.to_string(), "#228be680");
        assert_eq!(color.opaque().to_string(), "#228be6");
    }

    #[test]
    fn hsl_and_hsv_agree_on_the_extremes() {
        assert_eq!(ColorCode::hsl(0.0, 0.0, 1.0).to_hex(), "#ffffff");
        assert_eq!(ColorCode::hsl(0.0, 1.0, 0.5).to_hex(), "#ff0000");
        assert_eq!(ColorCode::hsl(0.0, 0.0, 0.0).to_hex(), "#000000");
        assert_eq!(ColorCode::hex(0x228be6).to_hex(), "#228be6");
        assert_eq!(ColorCode::from(HexColor::new(0x228be6)).to_hex(), "#228be6");
    }

    #[test]
    fn out_of_range_inputs_are_wrapped_or_clamped() {
        let color = ColorCode::hsva(-30.0, 2.0, -1.0, f64::NAN);
        assert_eq!(color.hue(), 330.0);
        assert_eq!(
            (color.saturation(), color.value(), color.alpha()),
            (1.0, 0.0, 0.0)
        );
    }

    /// Todo 2436: a `NaN` hue reached the hue slider's `aria-valuenow`.
    #[test]
    fn a_non_finite_hue_is_zero() {
        for hue in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(ColorCode::hsva(hue, 1.0, 1.0, 1.0).hue(), 0.0, "{hue}");
        }
    }
}
