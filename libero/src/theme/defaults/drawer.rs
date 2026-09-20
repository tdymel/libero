use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{Size, SizeCss, Sizes};

pub const DRAWER_SIZE: SizeCss = SizeCss::new("--lsx-drawer-size-");

/// Theme defaults for `Drawer`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawerDefaults {
    pub size: Size,
    pub sizes: Sizes<u16>,
}

impl DrawerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(200, 240, 280, 320, 400, 480),
    };
}

impl ToCssDeclarations for DrawerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.sizes.to_css_declarations(DRAWER_SIZE, "px")
    }
}
