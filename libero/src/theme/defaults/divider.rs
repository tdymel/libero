use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const DIVIDER_SPACING: CssVar = CssVar::new("--lsx-divider-spacing");
pub const DIVIDER_THICKNESS: SizeCss = SizeCss::new("--lsx-divider-thickness-");
pub const DIVIDER_LINE: CssVar = CssVar::new("--lsx-divider-line");

/// Theme defaults for `Divider`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DividerDefaults {
    /// The line's thickness step when `size` is omitted.
    pub size: Size,
    pub spacing: Option<Size>,
    pub thicknesses: Sizes<u16>,
}

impl DividerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Xs,
        spacing: None,
        thicknesses: Sizes::new(1, 2, 3, 4, 5, 6),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(DIVIDER_LINE, DIVIDER_THICKNESS.value(size))
    }

    pub fn horizontal_sx() -> Sx {
        sx().margin_top(DIVIDER_SPACING.value())
            .margin_bottom(DIVIDER_SPACING.value())
    }

    pub fn vertical_sx() -> Sx {
        sx().margin_left(DIVIDER_SPACING.value())
            .margin_right(DIVIDER_SPACING.value())
    }
}

impl ToCssDeclarations for DividerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let spacing = match self.spacing {
            Some(size) => SizeCss::SPACING.value(size),
            None => "0".to_string(),
        };
        let mut declarations = self
            .thicknesses
            .to_css_declarations(DIVIDER_THICKNESS, "px");
        declarations.push(DIVIDER_SPACING.declare(spacing));
        declarations
    }
}
