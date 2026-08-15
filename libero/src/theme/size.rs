#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Size {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl Size {
    pub(crate) const ALL: [Size; 5] = [Self::Xs, Self::Sm, Self::Md, Self::Lg, Self::Xl];

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    pub fn parse_dynamic(value: &str) -> Option<Self> {
        match value {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            _ => None,
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
