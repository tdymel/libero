use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const DATA_LIST_GAP: SizeCss = SizeCss::new("--lsx-data-list-gap-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataListDefaults {
    pub size: Size,
    pub gap: Sizes<u8>,
}

impl DataListDefaults {
    fn size_sx(size: Size) -> Sx {
        sx().gap(DATA_LIST_GAP.value(size))
    }

    pub fn theme_vars() -> Sx {
        Size::ALL.into_iter().fold(sx(), |base, size| {
            base.when(size.state_name(), Self::size_sx(size))
        })
    }
}

impl ToCssDeclarations for DataListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.gap.to_css_declarations(DATA_LIST_GAP, "px")
    }
}
