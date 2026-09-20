use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::{Size, SizeCss, Sizes};

pub const SIDEBAR_SIZE: SizeCss = SizeCss::new("--lsx-sidebar-size-");

str_enum! {
    /// Which edge a `Sidebar` borders. Descriptive only: it must match the
    /// panel's DOM position. Logical: `Start` is the right under `dir="rtl"`.
    #[state_prefix = "side"]
    pub enum SidebarSide {
        #[default]
        Start = "start",
        End = "end",
        Top = "top",
        Bottom = "bottom",
    }
}

/// Theme defaults for `Sidebar`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarDefaults {
    pub size: Size,
    pub sizes: Sizes<u16>,
}

impl SidebarDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(200, 240, 280, 320, 400, 480),
    };
}

impl ToCssDeclarations for SidebarDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        self.sizes.to_css_declarations(SIDEBAR_SIZE, "px")
    }
}
