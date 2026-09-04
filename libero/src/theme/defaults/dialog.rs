use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

pub const DIALOG_SIZE: SizeCss = SizeCss::new("--lsx-dialog-size-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogDefaults {
    pub size: Sizes<u16>,
}

impl DialogDefaults {
    pub const DEFAULT: Self = Self {
        size: Sizes::new(240, 300, 510, 600, 750, 900),
    };
}

impl ToCssDeclarations for DialogDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(DIALOG_SIZE, "px")
    }
}
