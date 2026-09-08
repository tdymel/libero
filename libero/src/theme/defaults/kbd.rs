use super::MONO_FONT_FAMILY;
use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const KBD_FONT_SIZE: SizeCss = SizeCss::new("--lsx-kbd-font-size-");

pub const KBD_FONT_FAMILY: CssVar = CssVar::new("--lsx-kbd-font-family");
pub const KBD_BACKGROUND: CssVar = CssVar::new("--lsx-kbd-background");
pub const KBD_BORDER: CssVar = CssVar::new("--lsx-kbd-border");
pub const KBD_COLOR: CssVar = CssVar::new("--lsx-kbd-color");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KbdDefaults {
    pub size: Size,
    pub font_sizes: Sizes<u16>,
    pub font_family: &'static str,
    pub background: &'static str,
    pub border: &'static str,
    pub color: &'static str,
}

impl KbdDefaults {
    pub const DEFAULT: Self = Self {
        // Matches Mantine's own default.
        size: Size::Sm,
        font_sizes: Sizes::new(10, 12, 14, 16, 20, 24),
        font_family: MONO_FONT_FAMILY,
        background: "#f6f8fa",
        border: "#d0d7de",
        color: "#57606a",
    };

    /// The same key on an inked page.
    pub const DARK: Self = Self {
        background: "#21262d",
        border: "#30363d",
        color: "#c9d1d9",
        ..Self::DEFAULT
    };

    fn size_sx(size: Size) -> Sx {
        sx().font_size(KBD_FONT_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        let border = format!("1px solid {}", KBD_BORDER.value());
        let border_bottom = format!("3px solid {}", KBD_BORDER.value());

        let base = sx()
            .font_family(KBD_FONT_FAMILY.value())
            .background(KBD_BACKGROUND.value())
            .color(KBD_COLOR.value())
            .border_top(border.clone())
            .border_left(border.clone())
            .border_right(border)
            // A touch thicker than the other 3 sides - reads as a keycap
            // with some depth instead of a flat pill.
            .border_bottom(border_bottom)
            .border_radius(SizeCss::RADIUS.value(Size::Sm));

        base.per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for KbdDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.font_sizes.to_css_declarations(KBD_FONT_SIZE, "px");
        declarations.push(KBD_FONT_FAMILY.declare(self.font_family));
        declarations.push(KBD_BACKGROUND.declare(self.background));
        declarations.push(KBD_BORDER.declare(self.border));
        declarations.push(KBD_COLOR.declare(self.color));
        declarations
    }
}
