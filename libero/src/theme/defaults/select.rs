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

impl SelectSizeLevel {
    pub const fn new(
        font_size: &'static str,
        height: &'static str,
        padding_x: &'static str,
    ) -> Self {
        Self {
            font_size,
            height,
            padding_x,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectDefaults {
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<SelectSizeLevel>,
}

impl SelectDefaults {
    pub const fn new(size: Size, radius: Size, sizes: Sizes<SelectSizeLevel>) -> Self {
        Self {
            size,
            radius,
            sizes,
        }
    }

    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(SELECT_FONT_SIZE.value(size))
            .height(SELECT_HEIGHT.value(size))
            .padding_left(SELECT_PADDING_X.value(size))
            .padding_right(SELECT_PADDING_X.value(size))
    }

    // Radius reads off the shared global radius scale rather than its own
    // - it's keyed by its own `radius-{size}` token (not `size-{size}`) so
    // it can be set independently of the select's own `size`.
    pub fn radius_sx(radius: Size) -> Sx {
        sx().border_radius(SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        let base = Size::ALL.into_iter().fold(sx(), |base, size| {
            base.when(size.state_name(), Self::size_sx(size))
        });
        Size::ALL.into_iter().fold(base, |base, radius| {
            base.when(radius.radius_state_name(), Self::radius_sx(radius))
        })
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
