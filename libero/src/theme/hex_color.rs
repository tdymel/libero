use crate::common::ConstStr;

use super::ColorShade;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub const fn shade(self, shade: ColorShade) -> Self {
        let percent = match shade {
            ColorShade::S1 => 80,
            ColorShade::S2 => 65,
            ColorShade::S3 => 50,
            ColorShade::S4 => 35,
            ColorShade::S5 => 20,
            ColorShade::S6 => 10,
            ColorShade::S7 => 0,
            ColorShade::S8 => -10,
            ColorShade::S9 => -20,
        };

        if percent >= 0 {
            self.mix(HexColor::new(0xFF_FF_FF), percent as u8)
        } else {
            self.mix(HexColor::new(0x00_00_00), (-percent) as u8)
        }
    }

    pub const fn contrast(self) -> Self {
        if self.luminance() >= 140 {
            HexColor::new(0x00_00_00)
        } else {
            HexColor::new(0xFF_FF_FF)
        }
    }

    pub const fn push_hex(self, mut css: ConstStr) -> ConstStr {
        css = css.push_char('#');
        css = css.push_char(hex_digit(self.r() >> 4));
        css = css.push_char(hex_digit(self.r() & 0x0F));
        css = css.push_char(hex_digit(self.g() >> 4));
        css = css.push_char(hex_digit(self.g() & 0x0F));
        css = css.push_char(hex_digit(self.b() >> 4));
        css = css.push_char(hex_digit(self.b() & 0x0F));
        css
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

const fn mix_channel(base: u8, other: u8, weight: u8) -> u8 {
    (((base as u16 * (100 - weight) as u16) + (other as u16 * weight as u16)) / 100) as u8
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + (value - 10)) as char,
        _ => panic!("invalid hex digit"),
    }
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
        const SHADE: HexColor = COLOR.shade(ColorShade::S1);
        const CONTRAST: HexColor = COLOR.contrast();

        assert_eq!(SHADE.rgb(), 0xD2_E7_FA);
        assert_eq!(CONTRAST.rgb(), 0xFF_FF_FF);
    }
}
