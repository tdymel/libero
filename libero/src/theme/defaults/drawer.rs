use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawerDefaults {
    pub size: Sizes<u16>,
}

impl DrawerDefaults {
    pub const fn new(size: Sizes<u16>) -> Self {
        Self { size }
    }
}

impl ToCssDeclarations for DrawerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(SizeCss::DRAWER_SIZE, "px")
    }
}
