use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, SizeCss, Sizes};

pub const HEADER_HEIGHT: SizeCss = SizeCss::new("--lsx-header-height-");
/// The page banner's height, published on `:root` for whatever sits below it.
pub const HEADER_HEIGHT_VAR: CssVar = CssVar::new("--lsx-header-height");

/// Theme defaults for `Header`, set on [`Theme`](crate::theme::Theme).
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
