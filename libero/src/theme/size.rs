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
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }
}
