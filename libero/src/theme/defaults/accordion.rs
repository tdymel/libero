use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, SizeCss, Sizes};

pub const ACCORDION_FONT_SIZE: SizeCss = SizeCss::new("--lsx-accordion-font-size-");
pub const ACCORDION_PADDING_X: SizeCss = SizeCss::new("--lsx-accordion-padding-x-");
pub const ACCORDION_PADDING_Y: SizeCss = SizeCss::new("--lsx-accordion-padding-y-");
pub const ACCORDION_CHEVRON: SizeCss = SizeCss::new("--lsx-accordion-chevron-");

// The picked level, resolved on the root so the triggers and panel bodies inherit it.
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

/// Theme defaults for `Accordion`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccordionDefaults {
    pub size: Size,
    pub sizes: Sizes<AccordionSizeLevel>,
    /// The line between two sections.
    pub border_color: ColorValue,
    pub hover_color: ColorValue,
    /// Milliseconds for the chevron's turn; the panel's height uses `theme.collapse.duration`.
    pub chevron_duration: u32,
}

impl AccordionDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(
            AccordionSizeLevel {
                font_size: "12px",
                padding_x: "8px",
                padding_y: "6px",
                chevron: "14px",
            },
            AccordionSizeLevel {
                font_size: "14px",
                padding_x: "12px",
                padding_y: "8px",
                chevron: "16px",
            },
            AccordionSizeLevel {
                font_size: "16px",
                padding_x: "16px",
                padding_y: "12px",
                chevron: "18px",
            },
            AccordionSizeLevel {
                font_size: "18px",
                padding_x: "20px",
                padding_y: "14px",
                chevron: "20px",
            },
            AccordionSizeLevel {
                font_size: "20px",
                padding_x: "24px",
                padding_y: "16px",
                chevron: "22px",
            },
            AccordionSizeLevel {
                font_size: "24px",
                padding_x: "28px",
                padding_y: "20px",
                chevron: "24px",
            },
        ),
        border_color: ColorValue::Shade(Color::Muted, ColorShade::S3),
        hover_color: ColorValue::Shade(Color::Muted, ColorShade::S1),
        chevron_duration: 150,
    };

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
