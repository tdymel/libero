use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use super::SANS_FONT_FAMILY;
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const TITLE_FONT_FAMILY: CssVar = CssVar::new("--lsx-title-font-family");

pub const TITLE_FONT_WEIGHT: SizeCss = SizeCss::new("--lsx-title-font-weight-");
pub const TITLE_FONT_SIZE: SizeCss = SizeCss::new("--lsx-title-font-size-");
pub const TITLE_LETTER_SPACING: SizeCss = SizeCss::new("--lsx-title-letter-spacing-");
pub const TITLE_LINE_HEIGHT: SizeCss = SizeCss::new("--lsx-title-line-height-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleSizeLevel {
    pub font_weight: &'static str,
    pub font_size: &'static str,
    pub letter_spacing: &'static str,
    pub line_height: &'static str,
}

/// Theme defaults for `Title`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleDefaults {
    /// The look only: a bare `Title` stays h1; the tag follows `component` or a passed `size`.
    pub size: Size,
    pub font_family: &'static str,
    pub sizes: Sizes<TitleSizeLevel>,
}

impl TitleDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Xxl,
        font_family: SANS_FONT_FAMILY,
        sizes: Sizes::new(
            TitleSizeLevel {
                font_weight: "400",
                font_size: "0.75rem",
                letter_spacing: "0em",
                line_height: "1.5",
            }, // xs (h6)
            TitleSizeLevel {
                font_weight: "400",
                font_size: "0.875rem",
                letter_spacing: "0em",
                line_height: "1.5",
            }, // sm (h5)
            TitleSizeLevel {
                font_weight: "400",
                font_size: "1rem",
                letter_spacing: "0em",
                line_height: "1.45",
            }, // md (h4)
            TitleSizeLevel {
                font_weight: "400",
                font_size: "1.375rem",
                letter_spacing: "0em",
                line_height: "1.4",
            }, // lg (h3)
            TitleSizeLevel {
                font_weight: "400",
                font_size: "1.625rem",
                letter_spacing: "-0.005em",
                line_height: "1.35",
            }, // xl (h2)
            TitleSizeLevel {
                font_weight: "400",
                font_size: "2.125rem",
                letter_spacing: "-0.01em",
                line_height: "1.3",
            }, // xxl (h1, default)
        ),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TITLE_FONT_SIZE.value(size))
            .font_weight(TITLE_FONT_WEIGHT.value(size))
            .letter_spacing(TITLE_LETTER_SPACING.value(size))
            .line_height(TITLE_LINE_HEIGHT.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().font_family(TITLE_FONT_FAMILY.value())
            .per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for TitleDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = vec![TITLE_FONT_FAMILY.declare(self.font_family)];
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(TITLE_FONT_WEIGHT.declare(size, level.font_weight));
            declarations.push(TITLE_FONT_SIZE.declare(size, level.font_size));
            declarations.push(TITLE_LETTER_SPACING.declare(size, level.letter_spacing));
            declarations.push(TITLE_LINE_HEIGHT.declare(size, level.line_height));
        }
        declarations
    }
}
