use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const HEADER_HEIGHT: SizeCss = SizeCss::new("--lsx-header-height-");
/// The page banner's height, published on `:root` for whatever sits below it.
pub const HEADER_HEIGHT_VAR: CssVar = CssVar::new("--lsx-header-height");
/// The height a header takes with no `size`; `HEADER_HEIGHT_VAR` is taken by the published one.
pub(crate) const HEADER_DEFAULT_HEIGHT: CssVar = CssVar::new("--lsx-header-height-default");

/// Theme defaults for `Header`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeaderDefaults {
    /// The height step when `size` is omitted.
    pub size: Size,
    pub heights: Sizes<u16>,
}

impl HeaderDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        heights: Sizes::new(48, 56, 64, 72, 80, 88),
    };
}

impl ToCssDeclarations for HeaderDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.heights.to_css_declarations(HEADER_HEIGHT, "px");
        declarations.push(HEADER_DEFAULT_HEIGHT.declare(HEADER_HEIGHT.value(self.size)));
        declarations
    }
}
