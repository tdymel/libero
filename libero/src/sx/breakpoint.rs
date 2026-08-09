#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    XS,
    S,
    M,
    L,
    XL,
}

impl Breakpoint {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::XS => "xs",
            Self::S => "s",
            Self::M => "m",
            Self::L => "l",
            Self::XL => "xl",
        }
    }

    pub const fn value(&self) -> &'static str {
        match self {
            Self::XS => "36em",
            Self::S => "48em",
            Self::M => "62em",
            Self::L => "75em",
            Self::XL => "88em",
        }
    }
}
