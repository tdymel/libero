use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::CssVar;

pub const FLOAT_Z_INDEX: CssVar = CssVar::new("--lsx-float-z-index");
pub const FLOAT_OFFSET_X: CssVar = CssVar::new("--lsx-float-offset-x");
pub const FLOAT_OFFSET_Y: CssVar = CssVar::new("--lsx-float-offset-y");

/// Anchor corner/edge within the floated element's `position: relative` parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    TopStart,
    TopCenter,
    TopEnd,
    CenterStart,
    CenterCenter,
    CenterEnd,
    BottomStart,
    BottomCenter,
    BottomEnd,
}

impl Default for Placement {
    fn default() -> Self {
        Self::CenterCenter
    }
}

impl From<&str> for Placement {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "top-start" => Self::TopStart,
            "top-center" => Self::TopCenter,
            "top-end" => Self::TopEnd,
            "center-start" => Self::CenterStart,
            "center-end" => Self::CenterEnd,
            "bottom-start" => Self::BottomStart,
            "bottom-center" => Self::BottomCenter,
            "bottom-end" => Self::BottomEnd,
            _ => Self::CenterCenter,
        }
    }
}

impl From<String> for Placement {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
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
