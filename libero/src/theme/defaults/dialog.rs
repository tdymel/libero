use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DialogDefaults {
    pub size: Sizes<u16>,
}

impl DialogDefaults {
    pub const fn new(size: Sizes<u16>) -> Self {
        Self { size }
    }
}

impl ToCssDeclarations for DialogDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(SizeCss::DIALOG_SIZE, "px")
    }
}
