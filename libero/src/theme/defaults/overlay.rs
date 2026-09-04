use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const OVERLAY_OPACITY: CssVar = CssVar::new("--lsx-overlay-opacity");
pub const OVERLAY_BLUR: CssVar = CssVar::new("--lsx-overlay-blur");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayDefaults {
    /// Alpha of the black dim behind the overlay's content.
    pub opacity: f32,
    /// A `backdrop-filter` value - `"none"` for no blur.
    pub blur: &'static str,
}

impl OverlayDefaults {
    pub const DEFAULT: Self = Self {
        opacity: 0.6,
        blur: "none",
    };
}

impl ToCssDeclarations for OverlayDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            OVERLAY_OPACITY.declare(self.opacity.to_string()),
            OVERLAY_BLUR.declare(self.blur),
        ]
    }
}
