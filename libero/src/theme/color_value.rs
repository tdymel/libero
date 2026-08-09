use super::{Color, ColorShade};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorValue {
    Shade(Color, ColorShade),
    Contrast(Color),
}

impl ColorValue {
    pub(crate) const fn parse(value: &'static str) -> Option<Self> {
        if starts_with(value, "primary") {
            return Some(Self::Shade(Color::Primary, parse_shade(value, 7)));
        }

        if starts_with(value, "secondary") {
            return Some(Self::Shade(Color::Secondary, parse_shade(value, 9)));
        }

        None
    }
}

const fn starts_with(value: &str, prefix: &str) -> bool {
    let value = value.as_bytes();
    let prefix = prefix.as_bytes();

    if prefix.len() > value.len() {
        return false;
    }

    let mut i = 0;
    while i < prefix.len() {
        if value[i] != prefix[i] {
            return false;
        }
        i += 1;
    }

    true
}

const fn parse_shade(value: &'static str, palette_len: usize) -> ColorShade {
    let bytes = value.as_bytes();
    if bytes.len() == palette_len {
        return ColorShade::S5;
    }

    if bytes.len() == palette_len + 2 && bytes[palette_len] == b'.' {
        let shade = bytes[palette_len + 1];
        return match shade {
            b'1' => ColorShade::S1,
            b'2' => ColorShade::S2,
            b'3' => ColorShade::S3,
            b'4' => ColorShade::S4,
            b'5' => ColorShade::S5,
            b'6' => ColorShade::S6,
            b'7' => ColorShade::S7,
            b'8' => ColorShade::S8,
            b'9' => ColorShade::S9,
            _ => panic!("invalid palette token"),
        };
    }

    panic!("invalid palette token")
}
