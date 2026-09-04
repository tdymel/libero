use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::Sx;
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, TextDefaults};

pub const ANCHOR_COLOR: CssVar = CssVar::new("--lsx-anchor-color");

str_enum! {
    /// When an `Anchor` draws its underline.
    #[state_prefix = "underline"]
    pub enum AnchorUnderline {
        Always = "always",
        #[default]
        Hover = "hover",
        Never = "never",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorDefaults {
    pub size: Size,
    pub underline: AnchorUnderline,
    pub color: Color,
}

impl AnchorDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        underline: AnchorUnderline::Hover,
        color: Color::Primary,
    };

    /// Text's own sizing (an `Anchor` is a `Text` that happens to link),
    /// plus the link color.
    pub fn theme_vars() -> Sx {
        TextDefaults::theme_vars().color(ANCHOR_COLOR.value())
    }
}

impl ToCssDeclarations for AnchorDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![ANCHOR_COLOR.declare(ColorValue::Shade(self.color, ColorShade::DEFAULT).value())]
    }
}
