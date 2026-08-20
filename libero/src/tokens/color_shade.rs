use crate::utils::warn;

/// Which mix curve `HexColor::shade` walks. A neutral grey needs a much wider
/// spread than a hue to cover the same perceptual range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShadeRamp {
    Chromatic,
    Neutral,
}

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
    /// What a bare `"primary"` resolves to: the base hex itself.
    pub(crate) const DEFAULT: Self = Self::S6;

    pub(crate) fn parse(suffix: Option<&str>) -> Self {
        match suffix {
            None | Some("") => Self::DEFAULT,
            Some("1") => Self::S1,
            Some("2") => Self::S2,
            Some("3") => Self::S3,
            Some("4") => Self::S4,
            Some("5") => Self::S5,
            Some("6") => Self::S6,
            Some("7") => Self::S7,
            Some("8") => Self::S8,
            Some("9") => Self::S9,
            Some(other) => {
                warn(&format!("unknown color shade `{other}`, falling back to 6"));
                Self::DEFAULT
            }
        }
    }

    pub(crate) const fn darker(self) -> Self {
        match self {
            Self::S1 => Self::S2,
            Self::S2 => Self::S3,
            Self::S3 => Self::S4,
            Self::S4 => Self::S5,
            Self::S5 => Self::S6,
            Self::S6 => Self::S7,
            Self::S7 => Self::S8,
            Self::S8 | Self::S9 => Self::S9,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn darker_steps_up_and_caps_at_s9() {
        assert_eq!(ColorShade::S6.darker(), ColorShade::S7);
        assert_eq!(ColorShade::S9.darker(), ColorShade::S9);
    }
}
