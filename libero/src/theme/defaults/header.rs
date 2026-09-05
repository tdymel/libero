use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

pub const HEADER_HEIGHT: SizeCss = SizeCss::new("--lsx-header-height-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderDefaults {
    pub heights: Sizes<u16>,
}

impl HeaderDefaults {
    pub const DEFAULT: Self = Self {
        heights: Sizes::new(48, 56, 64, 72, 80, 88),
    };
}

impl ToCssDeclarations for HeaderDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.heights.to_css_declarations(HEADER_HEIGHT, "px")
    }
}
