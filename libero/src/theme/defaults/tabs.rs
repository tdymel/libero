use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, Sizes};

pub const TABS_FONT_SIZE: SizeCss = SizeCss::new("--lsx-tabs-font-size-");
pub const TABS_PADDING_X: SizeCss = SizeCss::new("--lsx-tabs-padding-x-");
pub const TABS_PADDING_Y: SizeCss = SizeCss::new("--lsx-tabs-padding-y-");
pub const TABS_INDICATOR: SizeCss = SizeCss::new("--lsx-tabs-indicator-");
pub const TABS_ICON_GAP: SizeCss = SizeCss::new("--lsx-tabs-icon-gap-");

// The picked level, resolved on the tab list so the tab buttons - which carry no
// size `data-state` of their own - inherit it.
pub const TABS_PAD_X: CssVar = CssVar::new("--lsx-tabs-pad-x");
pub const TABS_PAD_Y: CssVar = CssVar::new("--lsx-tabs-pad-y");
pub const TABS_LINE: CssVar = CssVar::new("--lsx-tabs-line");
pub const TABS_GAP: CssVar = CssVar::new("--lsx-tabs-gap");

pub const TABS_BORDER_COLOR: CssVar = CssVar::new("--lsx-tabs-border-color");
pub const TABS_HOVER: CssVar = CssVar::new("--lsx-tabs-hover");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabsSizeLevel {
    pub font_size: &'static str,
    pub padding_x: &'static str,
    pub padding_y: &'static str,
    /// Thickness of the selected tab's underline.
    pub indicator: &'static str,
    /// Between a rich label's icon and its text.
    pub icon_gap: &'static str,
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
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(
            TabsSizeLevel {
                font_size: "12px",
                padding_x: "10px",
                padding_y: "6px",
                indicator: "2px",
                icon_gap: "6px",
            },
            TabsSizeLevel {
                font_size: "13px",
                padding_x: "12px",
                padding_y: "8px",
                indicator: "2px",
                icon_gap: "6px",
            },
            TabsSizeLevel {
                font_size: "14px",
                padding_x: "16px",
                padding_y: "10px",
                indicator: "2px",
                icon_gap: "8px",
            },
            TabsSizeLevel {
                font_size: "16px",
                padding_x: "20px",
                padding_y: "12px",
                indicator: "3px",
                icon_gap: "10px",
            },
            TabsSizeLevel {
                font_size: "18px",
                padding_x: "24px",
                padding_y: "14px",
                indicator: "3px",
                icon_gap: "12px",
            },
            TabsSizeLevel {
                font_size: "20px",
                padding_x: "28px",
                padding_y: "16px",
                indicator: "4px",
                icon_gap: "14px",
            },
        ),
        border_color: ColorValue::Shade(Color::Muted, ColorShade::S3),
        hover_color: ColorValue::Shade(Color::Muted, ColorShade::S1),
    };

    // Resolved on the tab list so each tab button inherits them.
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TABS_FONT_SIZE.value(size))
            .var(TABS_PAD_X, TABS_PADDING_X.value(size))
            .var(TABS_PAD_Y, TABS_PADDING_Y.value(size))
            .var(TABS_LINE, TABS_INDICATOR.value(size))
            .var(TABS_GAP, TABS_ICON_GAP.value(size))
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
            declarations.push(TABS_ICON_GAP.declare(size, level.icon_gap));
        }
        declarations
    }
}
