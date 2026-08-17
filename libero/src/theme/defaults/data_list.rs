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

    pub fn theme_vars() -> Sx {
        Size::ALL.into_iter().fold(sx(), |base, size| {
            base.when(size.state_name(), Self::size_sx(size))
        })
    }
}

impl ToCssDeclarations for DataListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.gap.to_css_declarations(SizeCss::DATA_LIST_GAP, "px")
    }
}
