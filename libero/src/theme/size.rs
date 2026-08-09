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
            Self::Xs => "36em",
            Self::Sm => "48em",
            Self::Md => "62em",
            Self::Lg => "75em",
            Self::Xl => "88em",
        }
    }
}
