use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const DIVIDER_SPACING: CssVar = CssVar::new("--lsx-divider-spacing");
pub const DIVIDER_THICKNESS: SizeCss = SizeCss::new("--lsx-divider-thickness-");
pub const DIVIDER_LINE: CssVar = CssVar::new("--lsx-divider-line");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DividerDefaults {
    pub spacing: Option<Size>,
    pub thickness: Sizes<u16>,
}

impl DividerDefaults {
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
        let mut declarations = self.thickness.to_css_declarations(DIVIDER_THICKNESS, "px");
        declarations.push(DIVIDER_SPACING.declare(spacing));
        declarations
    }
}
