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
pub struct TextSizeLevel {
    pub font_weight: &'static str,
    /// Any CSS length; by default the matching step of `Theme::font_size`.
    pub font_size: &'static str,
    pub letter_spacing: &'static str,
    pub line_height: &'static str,
}

/// Theme defaults for `Text`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextDefaults {
    pub size: Size,
    pub font_family: &'static str,
    pub sizes: Sizes<TextSizeLevel>,
}

impl TextDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        font_family: SANS_FONT_FAMILY,
        sizes: Sizes::new(
            TextSizeLevel {
                font_weight: "400",
                font_size: "var(--lsx-font-size-xs)",
                letter_spacing: "0em",
                line_height: "1.4",
            }, // xs
            TextSizeLevel {
                font_weight: "400",
                font_size: "var(--lsx-font-size-sm)",
                letter_spacing: "0em",
                line_height: "1.45",
            }, // sm
            TextSizeLevel {
                font_weight: "400",
                font_size: "var(--lsx-font-size-md)",
                letter_spacing: "0em",
                line_height: "1.5",
            }, // md (default)
            TextSizeLevel {
                font_weight: "400",
                font_size: "var(--lsx-font-size-lg)",
                letter_spacing: "0em",
                line_height: "1.55",
            }, // lg
            TextSizeLevel {
                font_weight: "400",
                font_size: "var(--lsx-font-size-xl)",
                letter_spacing: "0em",
                line_height: "1.6",
            }, // xl
            TextSizeLevel {
                font_weight: "400",
                font_size: "var(--lsx-font-size-xxl)",
                letter_spacing: "0em",
                line_height: "1.65",
            }, // xxl
        ),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(TEXT_FONT_SIZE.value(size))
            .font_weight(TEXT_FONT_WEIGHT.value(size))
            .letter_spacing(TEXT_LETTER_SPACING.value(size))
            .line_height(TEXT_LINE_HEIGHT.value(size))
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_default_size_reads_its_own_scale_step() {
        for size in Size::ALL {
            assert_eq!(
                TextDefaults::DEFAULT.sizes.get(size).font_size,
                SizeCss::FONT_SIZE.value(size)
            );
        }
    }
}
