use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const MENUBAR_GAP: CssVar = CssVar::new("--lsx-menubar-gap");
pub const MENUBAR_FONT_SIZE: SizeCss = SizeCss::new("--lsx-menubar-font-size-");
pub const MENUBAR_PADDING_X: SizeCss = SizeCss::new("--lsx-menubar-padding-x-");
pub const MENUBAR_PADDING_Y: SizeCss = SizeCss::new("--lsx-menubar-padding-y-");

// Resolved on the bar; the triggers, with no size `data-state`, inherit it.
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

/// Theme defaults for `Menubar`, set on [`Theme`](crate::theme::Theme).
/// The bar's row only: its dropdowns read [`MenuDefaults`](super::MenuDefaults).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenubarDefaults {
    pub size: Size,
    pub radius: Size,
    /// Between two triggers.
    pub gap: &'static str,
    pub sizes: Sizes<MenubarSizeLevel>,
    /// Whether the arrow keys wrap at the ends, along the bar and down each menu.
    pub loop_focus: bool,
}

impl MenubarDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        gap: "2px",
        sizes: Sizes::new(
            MenubarSizeLevel {
                font_size: "0.75rem",
                padding_x: "6px",
                padding_y: "2px",
            },
            MenubarSizeLevel {
                font_size: "0.8125rem",
                padding_x: "8px",
                padding_y: "3px",
            },
            MenubarSizeLevel {
                font_size: "0.875rem",
                padding_x: "10px",
                padding_y: "4px",
            },
            MenubarSizeLevel {
                font_size: "0.9375rem",
                padding_x: "12px",
                padding_y: "5px",
            },
            MenubarSizeLevel {
                font_size: "1rem",
                padding_x: "14px",
                padding_y: "6px",
            },
            MenubarSizeLevel {
                font_size: "1.0625rem",
                padding_x: "16px",
                padding_y: "7px",
            },
        ),
        loop_focus: true,
    };

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
