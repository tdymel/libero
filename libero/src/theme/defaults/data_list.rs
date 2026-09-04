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
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        gap: Sizes::new(6, 8, 12, 16, 20, 24),
    };

    fn size_sx(size: Size) -> Sx {
        sx().gap(DATA_LIST_GAP.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for DataListDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.gap.to_css_declarations(DATA_LIST_GAP, "px")
    }
}
