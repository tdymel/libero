use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogDefaults {
    pub size: Sizes<u16>,
}

impl ToCssDeclarations for DialogDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(SizeCss::DIALOG_SIZE, "px")
    }
}
