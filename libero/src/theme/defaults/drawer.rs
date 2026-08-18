use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{SizeCss, Sizes};

pub const DRAWER_SIZE: SizeCss = SizeCss::new("--lsx-drawer-size-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawerDefaults {
    pub size: Sizes<u16>,
}

impl ToCssDeclarations for DrawerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.size.to_css_declarations(DRAWER_SIZE, "px")
    }
}
