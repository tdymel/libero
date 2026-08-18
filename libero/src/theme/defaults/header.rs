use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderDefaults {
    pub height: Sizes<u16>,
}

impl ToCssDeclarations for HeaderDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.height
            .to_css_declarations(SizeCss::HEADER_HEIGHT, "px")
    }
}
