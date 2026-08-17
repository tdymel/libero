use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataListDefaults {
    pub size: Size,
    pub gap: Sizes<u8>,
}

impl DataListDefaults {
    pub const fn new(size: Size, gap: Sizes<u8>) -> Self {
        Self { size, gap }
    }

    fn size_sx(size: Size) -> Sx {
        sx().gap(SizeCss::DATA_LIST_GAP.value(size))
    }

    pub fn xs_sx() -> Sx {
        Self::size_sx(Size::Xs)
    }

    pub fn sm_sx() -> Sx {
        Self::size_sx(Size::Sm)
    }

    pub fn md_sx() -> Sx {
        Self::size_sx(Size::Md)
    }

    pub fn lg_sx() -> Sx {
        Self::size_sx(Size::Lg)
    }

    pub fn xl_sx() -> Sx {
        Self::size_sx(Size::Xl)
    }
}

impl ToCssDeclarations for DataListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.gap.to_css_declarations(SizeCss::DATA_LIST_GAP, "px")
    }
}
