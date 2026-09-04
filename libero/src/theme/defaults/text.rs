use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use super::SANS_FONT_FAMILY;
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const TEXT_FONT_FAMILY: CssVar = CssVar::new("--lsx-text-font-family");

pub const TEXT_FONT_WEIGHT: SizeCss = SizeCss::new("--lsx-text-font-weight-");
pub const TEXT_FONT_SIZE: SizeCss = SizeCss::new("--lsx-text-font-size-");
pub const TEXT_LETTER_SPACING: SizeCss = SizeCss::new("--lsx-text-letter-spacing-");
pub const TEXT_LINE_HEIGHT: SizeCss = SizeCss::new("--lsx-text-line-height-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSize {
    pub font_weight: &'static str,
    pub font_size: &'static str, // in rem
    pub letter_spacing: &'static str,
    pub line_height: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextDefaults {
    pub font_family: &'static str,
    pub sizes: Sizes<TextSize>,
}

impl TextDefaults {
    pub const DEFAULT: Self = Self {
        font_family: SANS_FONT_FAMILY,
        sizes: Sizes::new(
            TextSize {
                font_weight: "400",
                font_size: "0.75rem",
                letter_spacing: "0em",
                line_height: "1.4",
            }, // xs
            TextSize {
                font_weight: "400",
                font_size: "0.875rem",
                letter_spacing: "0em",
                line_height: "1.45",
            }, // sm
            TextSize {
                font_weight: "400",
                font_size: "1rem",
                letter_spacing: "0em",
                line_height: "1.5",
            }, // md (default)
            TextSize {
                font_weight: "400",
                font_size: "1.125rem",
                letter_spacing: "0em",
                line_height: "1.55",
            }, // lg
            TextSize {
                font_weight: "400",
                font_size: "1.25rem",
                letter_spacing: "0em",
                line_height: "1.6",
            }, // xl
            TextSize {
                font_weight: "400",
                font_size: "1.375rem",
                letter_spacing: "0em",
                line_height: "1.65",
            }, // xxl
        ),
    };

    /// Everything about a `Text` that varies by size. Font-family doesn't,
    /// hence `TEXT_FONT_FAMILY`.
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TEXT_FONT_SIZE.value(size))
            .font_weight(TEXT_FONT_WEIGHT.value(size))
            .letter_spacing(TEXT_LETTER_SPACING.value(size))
            .line_height(TEXT_LINE_HEIGHT.value(size))
    }

    /// Every size at once, each gated behind its own `data-state` selector,
    /// so all `Text`s share one static class and picking a size never mints
    /// a new one.
    pub fn theme_vars() -> Sx {
        sx().font_family(TEXT_FONT_FAMILY.value())
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for TextDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![TEXT_FONT_FAMILY.declare(self.font_family)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(TEXT_FONT_WEIGHT.declare(size, level.font_weight));
            declarations.push(TEXT_FONT_SIZE.declare(size, level.font_size));
            declarations.push(TEXT_LETTER_SPACING.declare(size, level.letter_spacing));
            declarations.push(TEXT_LINE_HEIGHT.declare(size, level.line_height));
        }
        declarations
    }
}
