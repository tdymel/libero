#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl Size {
    pub const fn parse(value: &'static str) -> Option<Self> {
        match value.as_bytes() {
            b"xs" => Some(Self::Xs),
            b"sm" => Some(Self::Sm),
            b"md" => Some(Self::Md),
            b"lg" => Some(Self::Lg),
            b"xl" => Some(Self::Xl),
            _ => None,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub const fn breakpoint_value(&self) -> &'static str {
        match self {
            Self::Xs => "36rem",
            Self::Sm => "48rem",
            Self::Md => "62rem",
            Self::Lg => "75rem",
            Self::Xl => "88rem",
        }
    }
}
