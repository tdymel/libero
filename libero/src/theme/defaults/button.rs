use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size};

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

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub xs: ButtonSizeLevel,
    pub sm: ButtonSizeLevel,
    pub md: ButtonSizeLevel,
    pub lg: ButtonSizeLevel,
    pub xl: ButtonSizeLevel,
}

impl ButtonDefaults {
    pub const fn new(
        size: Size,
        radius: Size,
        xs: ButtonSizeLevel,
        sm: ButtonSizeLevel,
        md: ButtonSizeLevel,
        lg: ButtonSizeLevel,
        xl: ButtonSizeLevel,
    ) -> Self {
        Self {
            size,
            radius,
            xs,
            sm,
            md,
            lg,
            xl,
        }
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
