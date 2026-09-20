use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::CssVar;

pub const ASPECT_RATIO: CssVar = CssVar::new("--lsx-aspect-ratio");

/// Theme defaults for `AspectRatio`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatioDefaults {
    pub ratio: f32,
}

impl AspectRatioDefaults {
    pub const DEFAULT: Self = Self { ratio: 1.0 };
}

impl ToCssDeclarations for AspectRatioDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![ASPECT_RATIO.declare(self.ratio.to_string())]
    }
}
