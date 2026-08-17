use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListDefaults {
    pub size: Size,
    pub gap: Sizes<u8>,
    pub indent: Sizes<u8>,
}

impl ListDefaults {
    pub const fn new(size: Size, gap: Sizes<u8>, indent: Sizes<u8>) -> Self {
        Self { size, gap, indent }
    }

    fn size_sx(size: Size) -> Sx {
        sx().gap(SizeCss::LIST_GAP.value(size))
            .selector("& ul", sx().padding_left(SizeCss::LIST_INDENT.value(size)))
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

    pub fn xxl_sx() -> Sx {
        Self::size_sx(Size::Xxl)
    }
}

impl ToCssDeclarations for ListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.gap.to_css_declarations(SizeCss::LIST_GAP, "px");
        declarations.extend(self.indent.to_css_declarations(SizeCss::LIST_INDENT, "px"));
        declarations
    }
}
