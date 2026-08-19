use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{Size, SizeCss, Sizes};

pub const SELECT_FONT_SIZE: SizeCss = SizeCss::new("--lsx-select-font-size-");
pub const SELECT_HEIGHT: SizeCss = SizeCss::new("--lsx-select-height-");
pub const SELECT_PADDING_X: SizeCss = SizeCss::new("--lsx-select-padding-x-");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectDefaults {
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<SelectSizeLevel>,
}

impl SelectDefaults {
    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(SELECT_FONT_SIZE.value(size))
            .height(SELECT_HEIGHT.value(size))
            .padding_left(SELECT_PADDING_X.value(size))
            .padding_right(SELECT_PADDING_X.value(size))
    }

    // The shared global radius scale, keyed by `radius-{size}` rather than
    // `size-{size}`, so it can be set independently of `size`.
    pub fn radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for SelectDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(SELECT_FONT_SIZE.declare(size, level.font_size));
            declarations.push(SELECT_HEIGHT.declare(size, level.height));
            declarations.push(SELECT_PADDING_X.declare(size, level.padding_x));
        }
        declarations
    }
}
