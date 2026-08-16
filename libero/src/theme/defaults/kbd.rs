use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, SizeCss, Sizes};

pub const KBD_FONT_FAMILY: CssVar = CssVar::new("--lsx-kbd-font-family");
pub const KBD_BACKGROUND: CssVar = CssVar::new("--lsx-kbd-background");
pub const KBD_BORDER: CssVar = CssVar::new("--lsx-kbd-border");
pub const KBD_COLOR: CssVar = CssVar::new("--lsx-kbd-color");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KbdDefaults {
    pub font_size: Sizes<u16>,
    pub font_family: &'static str,
    pub background: &'static str,
    pub border: &'static str,
    pub color: &'static str,
}

impl KbdDefaults {
    pub const fn new(
        font_size: Sizes<u16>,
        font_family: &'static str,
        background: &'static str,
        border: &'static str,
        color: &'static str,
    ) -> Self {
        Self {
            font_size,
            font_family,
            background,
            border,
            color,
        }
    }
}

impl ToCssDeclarations for KbdDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self
            .font_size
            .to_css_declarations(SizeCss::KBD_FONT_SIZE, "px");
        declarations.push(KBD_FONT_FAMILY.declare(self.font_family));
        declarations.push(KBD_BACKGROUND.declare(self.background));
        declarations.push(KBD_BORDER.declare(self.border));
        declarations.push(KBD_COLOR.declare(self.color));
        declarations
    }
}
