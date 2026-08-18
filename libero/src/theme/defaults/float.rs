use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::str_enum::str_enum;
use crate::theme::CssVar;

pub const FLOAT_Z_INDEX: CssVar = CssVar::new("--lsx-float-z-index");
pub const FLOAT_OFFSET_X: CssVar = CssVar::new("--lsx-float-offset-x");
pub const FLOAT_OFFSET_Y: CssVar = CssVar::new("--lsx-float-offset-y");

str_enum! {
    /// Anchor corner/edge within the floated element's `position: relative` parent.
    pub enum Placement {
        TopStart = "top-start",
        TopCenter = "top-center",
        TopEnd = "top-end",
        CenterStart = "center-start",
        #[default]
        CenterCenter = "center-center",
        CenterEnd = "center-end",
        BottomStart = "bottom-start",
        BottomCenter = "bottom-center",
        BottomEnd = "bottom-end",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FloatDefaults {
    pub z_index: u16,
    pub offset_x: &'static str,
    pub offset_y: &'static str,
    pub placement: Placement,
}

impl ToCssDeclarations for FloatDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            FLOAT_Z_INDEX.declare(self.z_index.to_string()),
            FLOAT_OFFSET_X.declare(self.offset_x),
            FLOAT_OFFSET_Y.declare(self.offset_y),
        ]
    }
}
