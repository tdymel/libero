use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes, Variant};

pub const CHIP_FONT_SIZE: SizeCss = SizeCss::new("--lsx-chip-font-size-");
pub const CHIP_HEIGHT: SizeCss = SizeCss::new("--lsx-chip-height-");
pub const CHIP_PADDING_X: SizeCss = SizeCss::new("--lsx-chip-padding-x-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipDefaults {
    /// The chrome a chip takes when a call site names none.
    pub variant: Variant,
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<ChipSizeLevel>,
}

impl ChipDefaults {
    pub const DEFAULT: Self = Self {
        variant: Variant::Filled,
        size: Size::Md,
        radius: Size::Xl,
        sizes: Sizes::new(
            ChipSizeLevel {
                font_size: "0.6875rem",
                height: "20px",
                padding_x: "8px",
            },
            ChipSizeLevel {
                font_size: "0.75rem",
                height: "24px",
                padding_x: "10px",
            },
            ChipSizeLevel {
                font_size: "0.8125rem",
                height: "28px",
                padding_x: "12px",
            },
            ChipSizeLevel {
                font_size: "0.875rem",
                height: "32px",
                padding_x: "14px",
            },
            ChipSizeLevel {
                font_size: "0.9375rem",
                height: "36px",
                padding_x: "16px",
            },
            ChipSizeLevel {
                font_size: "1rem",
                height: "40px",
                padding_x: "18px",
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
