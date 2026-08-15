use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const BUTTON_FONT_SIZE_XS: CssVar = CssVar::new("--lsx-button-font-size-xs");
pub const BUTTON_HEIGHT_XS: CssVar = CssVar::new("--lsx-button-height-xs");
pub const BUTTON_PADDING_X_XS: CssVar = CssVar::new("--lsx-button-padding-x-xs");

pub const BUTTON_FONT_SIZE_SM: CssVar = CssVar::new("--lsx-button-font-size-sm");
pub const BUTTON_HEIGHT_SM: CssVar = CssVar::new("--lsx-button-height-sm");
pub const BUTTON_PADDING_X_SM: CssVar = CssVar::new("--lsx-button-padding-x-sm");

pub const BUTTON_FONT_SIZE_MD: CssVar = CssVar::new("--lsx-button-font-size-md");
pub const BUTTON_HEIGHT_MD: CssVar = CssVar::new("--lsx-button-height-md");
pub const BUTTON_PADDING_X_MD: CssVar = CssVar::new("--lsx-button-padding-x-md");

pub const BUTTON_FONT_SIZE_LG: CssVar = CssVar::new("--lsx-button-font-size-lg");
pub const BUTTON_HEIGHT_LG: CssVar = CssVar::new("--lsx-button-height-lg");
pub const BUTTON_PADDING_X_LG: CssVar = CssVar::new("--lsx-button-padding-x-lg");

pub const BUTTON_FONT_SIZE_XL: CssVar = CssVar::new("--lsx-button-font-size-xl");
pub const BUTTON_HEIGHT_XL: CssVar = CssVar::new("--lsx-button-height-xl");
pub const BUTTON_PADDING_X_XL: CssVar = CssVar::new("--lsx-button-padding-x-xl");

pub const BUTTON_RADIUS: CssVar = CssVar::new("--lsx-button-radius");

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

    pub fn radius_sx() -> Sx {
        sx().border_radius(BUTTON_RADIUS.value())
    }

    pub fn xs_sx() -> Sx {
        sx().font_size(BUTTON_FONT_SIZE_XS.value())
            .height(BUTTON_HEIGHT_XS.value())
            .padding_left(BUTTON_PADDING_X_XS.value())
            .padding_right(BUTTON_PADDING_X_XS.value())
    }

    pub fn sm_sx() -> Sx {
        sx().font_size(BUTTON_FONT_SIZE_SM.value())
            .height(BUTTON_HEIGHT_SM.value())
            .padding_left(BUTTON_PADDING_X_SM.value())
            .padding_right(BUTTON_PADDING_X_SM.value())
    }

    pub fn md_sx() -> Sx {
        sx().font_size(BUTTON_FONT_SIZE_MD.value())
            .height(BUTTON_HEIGHT_MD.value())
            .padding_left(BUTTON_PADDING_X_MD.value())
            .padding_right(BUTTON_PADDING_X_MD.value())
    }

    pub fn lg_sx() -> Sx {
        sx().font_size(BUTTON_FONT_SIZE_LG.value())
            .height(BUTTON_HEIGHT_LG.value())
            .padding_left(BUTTON_PADDING_X_LG.value())
            .padding_right(BUTTON_PADDING_X_LG.value())
    }

    pub fn xl_sx() -> Sx {
        sx().font_size(BUTTON_FONT_SIZE_XL.value())
            .height(BUTTON_HEIGHT_XL.value())
            .padding_left(BUTTON_PADDING_X_XL.value())
            .padding_right(BUTTON_PADDING_X_XL.value())
    }
}

impl ToCssDeclarations for ButtonDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let vars = [
            (
                Size::Xs,
                BUTTON_FONT_SIZE_XS,
                BUTTON_HEIGHT_XS,
                BUTTON_PADDING_X_XS,
            ),
            (
                Size::Sm,
                BUTTON_FONT_SIZE_SM,
                BUTTON_HEIGHT_SM,
                BUTTON_PADDING_X_SM,
            ),
            (
                Size::Md,
                BUTTON_FONT_SIZE_MD,
                BUTTON_HEIGHT_MD,
                BUTTON_PADDING_X_MD,
            ),
            (
                Size::Lg,
                BUTTON_FONT_SIZE_LG,
                BUTTON_HEIGHT_LG,
                BUTTON_PADDING_X_LG,
            ),
            (
                Size::Xl,
                BUTTON_FONT_SIZE_XL,
                BUTTON_HEIGHT_XL,
                BUTTON_PADDING_X_XL,
            ),
        ];

        let mut declarations = vec![BUTTON_RADIUS.declare(SizeCss::RADIUS.value(self.radius))];
        for (size, font_size_var, height_var, padding_x_var) in vars {
            let level = self.sizes.get(size);
            declarations.push(font_size_var.declare(level.font_size));
            declarations.push(height_var.declare(level.height));
            declarations.push(padding_x_var.declare(level.padding_x));
        }
        declarations
    }
}
