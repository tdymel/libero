use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

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
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<ChipSizeLevel>,
}

impl ChipDefaults {
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
