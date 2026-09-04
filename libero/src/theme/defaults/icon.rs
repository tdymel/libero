use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

pub const ICON_SIZE: SizeCss = SizeCss::new("--lsx-icon-size-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconDefaults {
    pub size: Sizes<u16>,
}

impl IconDefaults {
    pub const DEFAULT: Self = Self {
        size: Sizes::new(16, 20, 24, 32, 40, 48),
    };
}

impl ToCssDeclarations for IconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(ICON_SIZE, "px")
    }
}
