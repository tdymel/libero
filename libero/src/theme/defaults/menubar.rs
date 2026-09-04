use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const MENUBAR_GAP: CssVar = CssVar::new("--lsx-menubar-gap");
pub const MENUBAR_FONT_SIZE: SizeCss = SizeCss::new("--lsx-menubar-font-size-");
pub const MENUBAR_PADDING_X: SizeCss = SizeCss::new("--lsx-menubar-padding-x-");
pub const MENUBAR_PADDING_Y: SizeCss = SizeCss::new("--lsx-menubar-padding-y-");

// The picked level, resolved on the bar so the triggers - which carry no size
// `data-state` of their own - inherit it. `Menu` does the same.
pub const MENUBAR_TRIGGER_FONT: CssVar = CssVar::new("--lsx-menubar-trigger-font");
pub const MENUBAR_TRIGGER_PAD_X: CssVar = CssVar::new("--lsx-menubar-trigger-pad-x");
pub const MENUBAR_TRIGGER_PAD_Y: CssVar = CssVar::new("--lsx-menubar-trigger-pad-y");
pub const MENUBAR_TRIGGER_RADIUS: CssVar = CssVar::new("--lsx-menubar-trigger-radius");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenubarSizeLevel {
    pub font_size: &'static str,
    pub padding_x: &'static str,
    pub padding_y: &'static str,
}

/// The bar's own row only - its dropdowns are `Menu`s and read
/// [`MenuDefaults`](super::MenuDefaults).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenubarDefaults {
    pub size: Size,
    pub radius: Size,
    /// Between two triggers.
    pub gap: &'static str,
    pub sizes: Sizes<MenubarSizeLevel>,
}

impl MenubarDefaults {
    pub fn size_sx(size: Size) -> Sx {
        sx().var(MENUBAR_TRIGGER_FONT, MENUBAR_FONT_SIZE.value(size))
            .var(MENUBAR_TRIGGER_PAD_X, MENUBAR_PADDING_X.value(size))
            .var(MENUBAR_TRIGGER_PAD_Y, MENUBAR_PADDING_Y.value(size))
    }

    pub fn radius_sx(radius: Size) -> Sx {
        sx().var(MENUBAR_TRIGGER_RADIUS, SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for MenubarDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![MENUBAR_GAP.declare(self.gap)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(MENUBAR_FONT_SIZE.declare(size, level.font_size));
            declarations.push(MENUBAR_PADDING_X.declare(size, level.padding_x));
            declarations.push(MENUBAR_PADDING_Y.declare(size, level.padding_y));
        }
        declarations
    }
}
