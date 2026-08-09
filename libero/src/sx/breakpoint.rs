use crate::theme::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    XS,
    S,
    M,
    L,
    XL,
}

impl Breakpoint {
    pub const fn size(&self) -> Size {
        match self {
            Self::XS => Size::Xs,
            Self::S => Size::Sm,
            Self::M => Size::Md,
            Self::L => Size::Lg,
            Self::XL => Size::Xl,
        }
    }

    pub const fn value(&self) -> &'static str {
        match self.size() {
            Size::Xs => "36em",
            Size::Sm => "48em",
            Size::Md => "62em",
            Size::Lg => "75em",
            Size::Xl => "88em",
        }
    }
}
