use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconDefaults {
    pub size: Sizes<u16>,
}

impl IconDefaults {
    pub const fn new(size: Sizes<u16>) -> Self {
        Self { size }
    }
}

impl ToCssDeclarations for IconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(SizeCss::ICON_SIZE, "px")
    }
}
