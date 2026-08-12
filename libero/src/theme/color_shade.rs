#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    pub(crate) fn parse(suffix: Option<&str>) -> Self {
        match suffix {
            None | Some("") => Self::S5,
            Some("1") => Self::S1,
            Some("2") => Self::S2,
            Some("3") => Self::S3,
            Some("4") => Self::S4,
            Some("5") => Self::S5,
            Some("6") => Self::S6,
            Some("7") => Self::S7,
            Some("8") => Self::S8,
            Some("9") => Self::S9,
            Some(_) => Self::S5,
        }
    }

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
}
