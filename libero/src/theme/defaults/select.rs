use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size};

pub const SELECT_FONT_SIZE_XS: CssVar = CssVar::new("--lsx-select-font-size-xs");
pub const SELECT_HEIGHT_XS: CssVar = CssVar::new("--lsx-select-height-xs");
pub const SELECT_PADDING_X_XS: CssVar = CssVar::new("--lsx-select-padding-x-xs");

pub const SELECT_FONT_SIZE_SM: CssVar = CssVar::new("--lsx-select-font-size-sm");
pub const SELECT_HEIGHT_SM: CssVar = CssVar::new("--lsx-select-height-sm");
pub const SELECT_PADDING_X_SM: CssVar = CssVar::new("--lsx-select-padding-x-sm");

pub const SELECT_FONT_SIZE_MD: CssVar = CssVar::new("--lsx-select-font-size-md");
pub const SELECT_HEIGHT_MD: CssVar = CssVar::new("--lsx-select-height-md");
pub const SELECT_PADDING_X_MD: CssVar = CssVar::new("--lsx-select-padding-x-md");

pub const SELECT_FONT_SIZE_LG: CssVar = CssVar::new("--lsx-select-font-size-lg");
pub const SELECT_HEIGHT_LG: CssVar = CssVar::new("--lsx-select-height-lg");
pub const SELECT_PADDING_X_LG: CssVar = CssVar::new("--lsx-select-padding-x-lg");

pub const SELECT_FONT_SIZE_XL: CssVar = CssVar::new("--lsx-select-font-size-xl");
pub const SELECT_HEIGHT_XL: CssVar = CssVar::new("--lsx-select-height-xl");
pub const SELECT_PADDING_X_XL: CssVar = CssVar::new("--lsx-select-padding-x-xl");

pub const SELECT_RADIUS: CssVar = CssVar::new("--lsx-select-radius");

#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub xs: SelectSizeLevel,
    pub sm: SelectSizeLevel,
    pub md: SelectSizeLevel,
    pub lg: SelectSizeLevel,
    pub xl: SelectSizeLevel,
}

impl SelectDefaults {
    pub const fn new(
        size: Size,
        radius: Size,
        xs: SelectSizeLevel,
        sm: SelectSizeLevel,
        md: SelectSizeLevel,
        lg: SelectSizeLevel,
        xl: SelectSizeLevel,
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

    pub fn radius_sx() -> Sx {
        sx().border_radius(SELECT_RADIUS.value())
    }

    pub fn xs_sx() -> Sx {
        sx().font_size(SELECT_FONT_SIZE_XS.value())
            .height(SELECT_HEIGHT_XS.value())
            .padding_left(SELECT_PADDING_X_XS.value())
            .padding_right(SELECT_PADDING_X_XS.value())
    }

    pub fn sm_sx() -> Sx {
        sx().font_size(SELECT_FONT_SIZE_SM.value())
            .height(SELECT_HEIGHT_SM.value())
            .padding_left(SELECT_PADDING_X_SM.value())
            .padding_right(SELECT_PADDING_X_SM.value())
    }

    pub fn md_sx() -> Sx {
        sx().font_size(SELECT_FONT_SIZE_MD.value())
            .height(SELECT_HEIGHT_MD.value())
            .padding_left(SELECT_PADDING_X_MD.value())
            .padding_right(SELECT_PADDING_X_MD.value())
    }

    pub fn lg_sx() -> Sx {
        sx().font_size(SELECT_FONT_SIZE_LG.value())
            .height(SELECT_HEIGHT_LG.value())
            .padding_left(SELECT_PADDING_X_LG.value())
            .padding_right(SELECT_PADDING_X_LG.value())
    }

    pub fn xl_sx() -> Sx {
        sx().font_size(SELECT_FONT_SIZE_XL.value())
            .height(SELECT_HEIGHT_XL.value())
            .padding_left(SELECT_PADDING_X_XL.value())
            .padding_right(SELECT_PADDING_X_XL.value())
    }
}
