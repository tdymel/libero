use std::fmt::{self, Display};

use super::{ColorShade, ShadeRamp};

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

    pub(crate) const fn contrast(self) -> Self {
        if self.luminance() >= 140 {
            HexColor::new(0x00_00_00)
        } else {
            HexColor::new(0xFF_FF_FF)
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
