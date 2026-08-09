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
}
