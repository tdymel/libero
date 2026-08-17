use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{Size, SizeCss, Sizes};

pub const BUTTON_FONT_SIZE: SizeCss = SizeCss::new("--lsx-button-font-size-");
pub const BUTTON_HEIGHT: SizeCss = SizeCss::new("--lsx-button-height-");
pub const BUTTON_PADDING_X: SizeCss = SizeCss::new("--lsx-button-padding-x-");

pub const BUTTON_RIPPLE_ANIMATION: &str = "lsx-ripple";
pub const BUTTON_RIPPLE_KEYFRAMES: &str =
    "@keyframes lsx-ripple{to{transform:translate(-50%, -50%) scale(1);opacity:0;}}";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonSizeLevel {
    pub font_size: &'static str,
    pub height: &'static str,
    pub padding_x: &'static str,
}

impl ButtonSizeLevel {
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
pub struct ButtonDefaults {
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<ButtonSizeLevel>,
}

impl ButtonDefaults {
    pub const fn new(size: Size, radius: Size, sizes: Sizes<ButtonSizeLevel>) -> Self {
        Self {
            size,
            radius,
            sizes,
        }
    }

    pub fn size_sx(size: Size) -> Sx {
        sx().font_size(BUTTON_FONT_SIZE.value(size))
            .height(BUTTON_HEIGHT.value(size))
            .padding_left(BUTTON_PADDING_X.value(size))
            .padding_right(BUTTON_PADDING_X.value(size))
    }

    // Radius reads off the shared global radius scale rather than its own -
    // it's keyed by its own `radius-{size}` token (not `size-{size}`) so it
    // can be set independently of the button's own `size`.
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

impl ToCssDeclarations for ButtonDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = Vec::new();
        for size in Size::ALL {
            let level = self.sizes.get(size);
            declarations.push(BUTTON_FONT_SIZE.declare(size, level.font_size));
            declarations.push(BUTTON_HEIGHT.declare(size, level.height));
            declarations.push(BUTTON_PADDING_X.declare(size, level.padding_x));
        }
        declarations
    }
}
