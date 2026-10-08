use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes, Variant};
use crate::tokens::Color;

pub const CHIP_FONT_SIZE: SizeCss = SizeCss::new("--lsx-chip-font-size-");
pub const CHIP_HEIGHT: SizeCss = SizeCss::new("--lsx-chip-height-");
pub const CHIP_PADDING_X: SizeCss = SizeCss::new("--lsx-chip-padding-x-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

/// Theme defaults for `Chip`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipDefaults {
    /// The chrome a chip takes when a call site names none.
    pub variant: Variant,
    pub size: Size,
    pub radius: Size,
    /// The colour a chip takes when a call site names none.
    pub color: Color,
    pub sizes: Sizes<ChipSizeLevel>,
}

impl ChipDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        size: Size::Md,
        radius: Size::Xl,
        color: Color::Primary,
        sizes: Sizes::new(
            // 24px like `sm`: the WCAG 2.5.8 minimum target (1494).
            ChipSizeLevel {
                font_size: "0.6875rem",
                height: "1.5rem",
                padding_x: "0.5rem",
            },
            ChipSizeLevel {
                font_size: "0.75rem",
                height: "1.5rem",
                padding_x: "0.625rem",
            },
            ChipSizeLevel {
                font_size: "0.8125rem",
                height: "1.75rem",
                padding_x: "0.75rem",
            },
            ChipSizeLevel {
                font_size: "0.875rem",
                height: "2rem",
                padding_x: "0.875rem",
            },
            ChipSizeLevel {
                font_size: "0.9375rem",
                height: "2.25rem",
                padding_x: "1rem",
            },
            ChipSizeLevel {
                font_size: "1rem",
                height: "2.5rem",
                padding_x: "1.125rem",
            },
        ),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(CHIP_FONT_SIZE.value(size))
            .height(CHIP_HEIGHT.value(size))
            .padding_left(CHIP_PADDING_X.value(size))
            .padding_right(CHIP_PADDING_X.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
            .per_radius(super::ButtonDefaults::radius_sx)
    }
}

impl ToCssDeclarations for ChipDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(CHIP_FONT_SIZE.declare(size, level.font_size));
            declarations.push(CHIP_HEIGHT.declare(size, level.height));
            declarations.push(CHIP_PADDING_X.declare(size, level.padding_x));
        }
        declarations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 741: a px box under `overflow: hidden` clips a rem label at 200% text size.
    #[test]
    fn every_box_length_follows_the_text_size() {
        for size in Size::ALL {
            let level = ChipDefaults::DEFAULT.sizes.get(size);
            for length in [level.font_size, level.height, level.padding_x] {
                assert!(length.ends_with("rem"), "{size:?}: {length}");
            }
        }
    }
}
