use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const CODE_FONT_FAMILY: CssVar = CssVar::new("--lsx-code-font-family");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodeDefaults {
    pub font_family: &'static str,
}

impl CodeDefaults {
    pub const fn new(font_family: &'static str) -> Self {
        Self { font_family }
    }
}

impl ToCssDeclarations for CodeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![CODE_FONT_FAMILY.declare(self.font_family)]
    }
}
