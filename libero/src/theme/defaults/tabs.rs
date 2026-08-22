use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ColorValue, CssVar, Size, SizeCss, Sizes};

pub const TABS_FONT_SIZE: SizeCss = SizeCss::new("--lsx-tabs-font-size-");
pub const TABS_PADDING_X: SizeCss = SizeCss::new("--lsx-tabs-padding-x-");
pub const TABS_PADDING_Y: SizeCss = SizeCss::new("--lsx-tabs-padding-y-");
pub const TABS_INDICATOR: SizeCss = SizeCss::new("--lsx-tabs-indicator-");

// The picked level, resolved on the tab list so the tab buttons - which carry no
// size `data-state` of their own - inherit it.
pub const TABS_PAD_X: CssVar = CssVar::new("--lsx-tabs-pad-x");
pub const TABS_PAD_Y: CssVar = CssVar::new("--lsx-tabs-pad-y");
pub const TABS_LINE: CssVar = CssVar::new("--lsx-tabs-line");

pub const TABS_BORDER_COLOR: CssVar = CssVar::new("--lsx-tabs-border-color");
pub const TABS_HOVER: CssVar = CssVar::new("--lsx-tabs-hover");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabsSizeLevel {
    pub font_size: &'static str,
    pub padding_x: &'static str,
    pub padding_y: &'static str,
    /// Thickness of the selected tab's underline.
    pub indicator: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabsDefaults {
    pub size: Size,
    pub sizes: Sizes<TabsSizeLevel>,
    /// The line the whole strip sits on.
    pub border_color: ColorValue,
    pub hover_color: ColorValue,
}

impl TabsDefaults {
    // Resolved on the tab list so each tab button inherits them.
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TABS_FONT_SIZE.value(size))
            .var(TABS_PAD_X, TABS_PADDING_X.value(size))
            .var(TABS_PAD_Y, TABS_PADDING_Y.value(size))
            .var(TABS_LINE, TABS_INDICATOR.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for TabsDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![
            TABS_BORDER_COLOR.declare(self.border_color.value()),
            TABS_HOVER.declare(self.hover_color.value()),
        ];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(TABS_FONT_SIZE.declare(size, level.font_size));
            declarations.push(TABS_PADDING_X.declare(size, level.padding_x));
            declarations.push(TABS_PADDING_Y.declare(size, level.padding_y));
            declarations.push(TABS_INDICATOR.declare(size, level.indicator));
        }
        declarations
    }
}
