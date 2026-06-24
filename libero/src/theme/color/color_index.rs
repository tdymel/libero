#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorIndex {
    C0,
    C1,
    C2,
    C3,
    C4,
    Main,
    C6,
    C7,
    C8,
    C9,
}

impl ColorIndex {
    pub const fn as_u8(self) -> u8 {
        match self {
            Self::C0 => 0,
            Self::C1 => 1,
            Self::C2 => 2,
            Self::C3 => 3,
            Self::C4 => 4,
            Self::Main => 5,
            Self::C6 => 6,
            Self::C7 => 7,
            Self::C8 => 8,
            Self::C9 => 9,
        }
    }

    pub const fn from_u8(index: u8) -> Self {
        match index {
            0 => Self::C0,
            1 => Self::C1,
            2 => Self::C2,
            3 => Self::C3,
            4 => Self::C4,
            5 => Self::Main,
            6 => Self::C6,
            7 => Self::C7,
            8 => Self::C8,
            9 => Self::C9,
            _ => panic!("ColorIndex must be between 0 and 9"),
        }
    }
}

pub const trait IntoColorIndex {
    fn into_color_index(self) -> ColorIndex;
}

impl const IntoColorIndex for ColorIndex {
    fn into_color_index(self) -> ColorIndex {
        self
    }
}

impl const IntoColorIndex for u8 {
    fn into_color_index(self) -> ColorIndex {
        ColorIndex::from_u8(self)
    }
}
