#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorShade {
    S1 = 1,
    S2 = 2,
    S3 = 3,
    S4 = 4,
    S5 = 5,
    S6 = 6,
    S7 = 7,
    S8 = 8,
    S9 = 9,
}

impl ColorShade {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::S1 => "1",
            Self::S2 => "2",
            Self::S3 => "3",
            Self::S4 => "4",
            Self::S5 => "5",
            Self::S6 => "6",
            Self::S7 => "7",
            Self::S8 => "8",
            Self::S9 => "9",
        }
    }

    pub const fn parse(value: &'static str, palette_len: usize) -> Self {
        let bytes = value.as_bytes();
        if bytes.len() == palette_len {
            return Self::S5;
        }

        if bytes.len() == palette_len + 2 && bytes[palette_len] == b'.' {
            let shade = bytes[palette_len + 1];
            return match shade {
                b'1' => Self::S1,
                b'2' => Self::S2,
                b'3' => Self::S3,
                b'4' => Self::S4,
                b'5' => Self::S5,
                b'6' => Self::S6,
                b'7' => Self::S7,
                b'8' => Self::S8,
                b'9' => Self::S9,
                _ => panic!("invalid palette token"),
            };
        }

        panic!("invalid palette token")
    }
}
