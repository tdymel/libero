use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss};

pub const DIVIDER_SPACING: CssVar = CssVar::new("--lsx-divider-spacing");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DividerDefaults {
    pub spacing: Option<Size>,
}

impl DividerDefaults {
    pub const fn new(spacing: Option<Size>) -> Self {
        Self { spacing }
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
        vec![DIVIDER_SPACING.declare(spacing)]
    }
}
