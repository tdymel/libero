use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::{SizeCss, Sizes};

pub const SIDEBAR_SIZE: SizeCss = SizeCss::new("--lsx-sidebar-size-");

str_enum! {
    /// Which edge a `Sidebar` borders and which axis its `size` applies to.
    /// Descriptive only - an in-flow panel is placed by its parent's layout,
    /// so this must match the DOM position you give it.
    #[state_prefix = "side"]
    pub enum SidebarSide {
        #[default]
        Left = "left",
        Right = "right",
        Top = "top",
        Bottom = "bottom",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarDefaults {
    pub sizes: Sizes<u16>,
}

impl SidebarDefaults {
    pub const DEFAULT: Self = Self {
        sizes: Sizes::new(200, 240, 280, 320, 400, 480),
    };
}

impl ToCssDeclarations for SidebarDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.sizes.to_css_declarations(SIDEBAR_SIZE, "px")
    }
}
