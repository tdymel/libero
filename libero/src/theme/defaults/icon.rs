use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Size, SizeCss, Sizes, Variant};

pub const ICON_SIZE: SizeCss = SizeCss::new("--lsx-icon-size-");
/// The size an icon takes with no `size`; the prop writes `ICON_SIZE`'s override over it.
pub(crate) const ICON_DEFAULT_SIZE: CssVar = CssVar::new("--lsx-icon-size");
pub(crate) const ICON_DEFAULT_RADIUS: CssVar = CssVar::new("--lsx-icon-radius-default");

/// Theme defaults for `Icon`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconDefaults {
    /// The chrome an icon takes when a call site names none.
    pub variant: Variant,
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<u16>,
}

impl IconDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        size: Size::Md,
        radius: Size::Sm,
        sizes: Sizes::new(16, 20, 24, 32, 40, 48),
    };
}

impl ToCssDeclarations for IconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.sizes.to_css_declarations(ICON_SIZE, "px");
        declarations.push(ICON_DEFAULT_SIZE.declare(ICON_SIZE.value(self.size)));
        declarations.push(ICON_DEFAULT_RADIUS.declare(SizeCss::RADIUS.value(self.radius)));
        declarations
    }
}
