use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const MENU_FONT_SIZE: SizeCss = SizeCss::new("--lsx-menu-font-size-");
pub const MENU_ITEM_HEIGHT: SizeCss = SizeCss::new("--lsx-menu-item-height-");
pub const MENU_PADDING_X: SizeCss = SizeCss::new("--lsx-menu-padding-x-");
pub const MENU_LABEL_FONT_SIZE: SizeCss = SizeCss::new("--lsx-menu-label-font-size-");
pub const MENU_MAX_HEIGHT: CssVar = CssVar::new("--lsx-menu-max-height");

// Resolved on the menu box; the items, with no size `data-state`, inherit it.
pub const MENU_ITEM_FONT: CssVar = CssVar::new("--lsx-menu-item-font");
pub const MENU_ITEM_MIN_HEIGHT: CssVar = CssVar::new("--lsx-menu-item-min-height");
pub const MENU_ITEM_PAD_X: CssVar = CssVar::new("--lsx-menu-item-pad-x");
pub const MENU_LABEL_FONT: CssVar = CssVar::new("--lsx-menu-label-font");
pub const MENU_ITEM_RADIUS: CssVar = CssVar::new("--lsx-menu-item-radius");

/// The menu box's padding: the inset an item nests at, and so its radius cut.
pub const MENU_PADDING: &str = "4px";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuSizeLevel {
    pub font_size: &'static str,
    /// An item's minimum height.
    pub item_height: f64,
    pub padding_x: &'static str,
    /// A group's name, above its items.
    pub label_font_size: &'static str,
}

/// Theme defaults for `Menu`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuDefaults {
    pub size: Size,
    pub radius: Size,
    pub max_height: &'static str,
    pub sizes: Sizes<MenuSizeLevel>,
    /// Milliseconds of hover before a submenu opens or a sibling takes over:
    /// long enough to cross a sibling on the way into a submenu.
    pub submenu_delay: u32,
    /// Whether choosing an item closes the menu.
    pub close_on_select: bool,
    /// Whether the arrow keys wrap from the last item to the first.
    pub loop_focus: bool,
}

impl MenuDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        radius: Size::Sm,
        max_height: "340px",
        sizes: Sizes::new(
            MenuSizeLevel {
                font_size: "0.75rem",
                item_height: 28.0,
                padding_x: "8px",
                label_font_size: "0.6875rem",
            },
            MenuSizeLevel {
                font_size: "0.8125rem",
                item_height: 32.0,
                padding_x: "10px",
                label_font_size: "0.75rem",
            },
            MenuSizeLevel {
                font_size: "0.875rem",
                item_height: 36.0,
                padding_x: "12px",
                label_font_size: "0.75rem",
            },
            MenuSizeLevel {
                font_size: "0.9375rem",
                item_height: 40.0,
                padding_x: "14px",
                label_font_size: "0.8125rem",
            },
            MenuSizeLevel {
                font_size: "1rem",
                item_height: 44.0,
                padding_x: "16px",
                label_font_size: "0.875rem",
            },
            MenuSizeLevel {
                font_size: "1.0625rem",
                item_height: 48.0,
                padding_x: "18px",
                label_font_size: "0.9375rem",
            },
        ),
        submenu_delay: 150,
        close_on_select: true,
        loop_focus: true,
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(MENU_ITEM_FONT, MENU_FONT_SIZE.value(size))
            .var(MENU_ITEM_MIN_HEIGHT, MENU_ITEM_HEIGHT.value(size))
            .var(MENU_ITEM_PAD_X, MENU_PADDING_X.value(size))
            .var(MENU_LABEL_FONT, MENU_LABEL_FONT_SIZE.value(size))
    }

    /// `MENU_PADDING` tighter than the box's corner, or it crosses it at `xxl`.
    pub fn radius_sx(radius: Size) -> Sx {
        sx().var(
            MENU_ITEM_RADIUS,
            format!(
                "max(0px, calc({} - {MENU_PADDING}))",
                SizeCss::RADIUS.value(radius)
            ),
        )
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for MenuDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![MENU_MAX_HEIGHT.declare(self.max_height)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(MENU_FONT_SIZE.declare(size, level.font_size));
            declarations.push(MENU_ITEM_HEIGHT.declare(size, format!("{}px", level.item_height)));
            declarations.push(MENU_PADDING_X.declare(size, level.padding_x));
            declarations.push(MENU_LABEL_FONT_SIZE.declare(size, level.label_font_size));
        }
        declarations
    }
}
