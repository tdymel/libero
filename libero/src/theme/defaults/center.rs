use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::CssVar;

pub const CENTER_DISPLAY: CssVar = CssVar::new("--lsx-center-display");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CenterDefaults {
    pub inline: bool,
}

impl CenterDefaults {
    pub const fn new(inline: bool) -> Self {
        Self { inline }
    }
}

impl ToCssDeclarations for CenterDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![CENTER_DISPLAY.declare(if self.inline { "inline-flex" } else { "flex" })]
    }
}
