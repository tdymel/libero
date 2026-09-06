use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes, Variant};

pub const ICON_SIZE: SizeCss = SizeCss::new("--lsx-icon-size-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconDefaults {
    /// The chrome an icon takes when a call site names none.
    pub variant: Variant,
    pub sizes: Sizes<u16>,
}

impl IconDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        sizes: Sizes::new(16, 20, 24, 32, 40, 48),
    };
}

impl ToCssDeclarations for IconDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.sizes.to_css_declarations(ICON_SIZE, "px")
    }
}
