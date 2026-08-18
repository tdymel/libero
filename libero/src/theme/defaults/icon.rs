use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconDefaults {
    pub size: Sizes<u16>,
}

impl ToCssDeclarations for IconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(SizeCss::ICON_SIZE, "px")
    }
}
