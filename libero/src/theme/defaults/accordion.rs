use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ColorValue, CssVar, Size, SizeCss, Sizes};

pub const ACCORDION_FONT_SIZE: SizeCss = SizeCss::new("--lsx-accordion-font-size-");
pub const ACCORDION_PADDING_X: SizeCss = SizeCss::new("--lsx-accordion-padding-x-");
pub const ACCORDION_PADDING_Y: SizeCss = SizeCss::new("--lsx-accordion-padding-y-");
pub const ACCORDION_CHEVRON: SizeCss = SizeCss::new("--lsx-accordion-chevron-");

// The picked level, resolved on the root so the triggers and panel bodies -
// which carry no size `data-state` of their own - inherit it.
pub const ACCORDION_PAD_X: CssVar = CssVar::new("--lsx-accordion-pad-x");
pub const ACCORDION_PAD_Y: CssVar = CssVar::new("--lsx-accordion-pad-y");
pub const ACCORDION_CHEVRON_SIZE: CssVar = CssVar::new("--lsx-accordion-chevron");

pub const ACCORDION_BORDER_COLOR: CssVar = CssVar::new("--lsx-accordion-border-color");
pub const ACCORDION_HOVER: CssVar = CssVar::new("--lsx-accordion-hover");
pub const ACCORDION_CHEVRON_DURATION: CssVar = CssVar::new("--lsx-accordion-chevron-duration");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccordionSizeLevel {
    pub font_size: &'static str,
    pub padding_x: &'static str,
    pub padding_y: &'static str,
    /// The chevron's box, width and height.
    pub chevron: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccordionDefaults {
    pub size: Size,
    pub sizes: Sizes<AccordionSizeLevel>,
    /// The line between two sections.
    pub border_color: ColorValue,
    pub hover_color: ColorValue,
    /// Milliseconds for the chevron's turn. The panel's own height animation is
    /// `theme.collapse.duration`, not this.
    pub chevron_duration: u32,
}

impl AccordionDefaults {
    // Resolved on the root so every trigger and panel body inherits them.
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(ACCORDION_FONT_SIZE.value(size))
            .var(ACCORDION_PAD_X, ACCORDION_PADDING_X.value(size))
            .var(ACCORDION_PAD_Y, ACCORDION_PADDING_Y.value(size))
            .var(ACCORDION_CHEVRON_SIZE, ACCORDION_CHEVRON.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for AccordionDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![
            ACCORDION_BORDER_COLOR.declare(self.border_color.value()),
            ACCORDION_HOVER.declare(self.hover_color.value()),
            ACCORDION_CHEVRON_DURATION.declare(format!("{}ms", self.chevron_duration)),
        ];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(ACCORDION_FONT_SIZE.declare(size, level.font_size));
            declarations.push(ACCORDION_PADDING_X.declare(size, level.padding_x));
            declarations.push(ACCORDION_PADDING_Y.declare(size, level.padding_y));
            declarations.push(ACCORDION_CHEVRON.declare(size, level.chevron));
        }
        declarations
    }
}
