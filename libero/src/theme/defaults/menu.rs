use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const MENU_FONT_SIZE: SizeCss = SizeCss::new("--lsx-menu-font-size-");
pub const MENU_ITEM_HEIGHT: SizeCss = SizeCss::new("--lsx-menu-item-height-");
pub const MENU_PADDING_X: SizeCss = SizeCss::new("--lsx-menu-padding-x-");
pub const MENU_LABEL_FONT_SIZE: SizeCss = SizeCss::new("--lsx-menu-label-font-size-");
pub const MENU_MAX_HEIGHT: CssVar = CssVar::new("--lsx-menu-max-height");

// The picked level, resolved on the menu box so the items - which carry no
// size `data-state` of their own - inherit it. `Tabs` does the same.
pub const MENU_ITEM_FONT: CssVar = CssVar::new("--lsx-menu-item-font");
pub const MENU_ITEM_MIN_HEIGHT: CssVar = CssVar::new("--lsx-menu-item-min-height");
pub const MENU_ITEM_PAD_X: CssVar = CssVar::new("--lsx-menu-item-pad-x");
pub const MENU_LABEL_FONT: CssVar = CssVar::new("--lsx-menu-label-font");
pub const MENU_ITEM_RADIUS: CssVar = CssVar::new("--lsx-menu-item-radius");

/// The menu box's own padding, and so the inset an item nests at - which is
/// what its corner radius has to be smaller by.
pub const MENU_PADDING: &str = "4px";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuSizeLevel {
    pub font_size: &'static str,
    /// An item's minimum height - a taller item grows past it.
    pub item_height: f64,
    pub padding_x: &'static str,
    /// A group's name, above its items.
    pub label_font_size: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuDefaults {
    pub size: Size,
    pub radius: Size,
    pub max_height: &'static str,
    pub sizes: Sizes<MenuSizeLevel>,
    /// Milliseconds the pointer has to rest on an item before its submenu
    /// opens, or before a sibling takes over from an open submenu. Long enough
    /// to cross a sibling on the way into a submenu without closing it.
    pub submenu_delay: u32,
}

impl MenuDefaults {
    pub fn size_sx(size: Size) -> Sx {
        sx().var(MENU_ITEM_FONT, MENU_FONT_SIZE.value(size))
            .var(MENU_ITEM_MIN_HEIGHT, MENU_ITEM_HEIGHT.value(size))
            .var(MENU_ITEM_PAD_X, MENU_PADDING_X.value(size))
            .var(MENU_LABEL_FONT, MENU_LABEL_FONT_SIZE.value(size))
    }

    /// An item nests `MENU_PADDING` inside the box, so its corner has to be
    /// that much tighter or it crosses the box's own - visibly at `xxl`.
    /// `max` keeps the small steps at 0 rather than negative.
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
