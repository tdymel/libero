use crate::sx::{Sx, sx};

use crate::theme::CssVar;

// Font family (shared across all sizes)
pub const TEXT_FONT_FAMILY: CssVar = CssVar::new("--lsx-text-font-family");

// XS
pub const TEXT_FONT_WEIGHT_XS: CssVar = CssVar::new("--lsx-text-font-weight-xs");
pub const TEXT_FONT_SIZE_XS: CssVar = CssVar::new("--lsx-text-font-size-xs");
pub const TEXT_LETTER_SPACING_XS: CssVar = CssVar::new("--lsx-text-letter-spacing-xs");
pub const TEXT_LINE_HEIGHT_XS: CssVar = CssVar::new("--lsx-text-line-height-xs");

// SM
pub const TEXT_FONT_WEIGHT_SM: CssVar = CssVar::new("--lsx-text-font-weight-sm");
pub const TEXT_FONT_SIZE_SM: CssVar = CssVar::new("--lsx-text-font-size-sm");
pub const TEXT_LETTER_SPACING_SM: CssVar = CssVar::new("--lsx-text-letter-spacing-sm");
pub const TEXT_LINE_HEIGHT_SM: CssVar = CssVar::new("--lsx-text-line-height-sm");

// MD
pub const TEXT_FONT_WEIGHT_MD: CssVar = CssVar::new("--lsx-text-font-weight-md");
pub const TEXT_FONT_SIZE_MD: CssVar = CssVar::new("--lsx-text-font-size-md");
pub const TEXT_LETTER_SPACING_MD: CssVar = CssVar::new("--lsx-text-letter-spacing-md");
pub const TEXT_LINE_HEIGHT_MD: CssVar = CssVar::new("--lsx-text-line-height-md");

// LG
pub const TEXT_FONT_WEIGHT_LG: CssVar = CssVar::new("--lsx-text-font-weight-lg");
pub const TEXT_FONT_SIZE_LG: CssVar = CssVar::new("--lsx-text-font-size-lg");
pub const TEXT_LETTER_SPACING_LG: CssVar = CssVar::new("--lsx-text-letter-spacing-lg");
pub const TEXT_LINE_HEIGHT_LG: CssVar = CssVar::new("--lsx-text-line-height-lg");

// XL
pub const TEXT_FONT_WEIGHT_XL: CssVar = CssVar::new("--lsx-text-font-weight-xl");
pub const TEXT_FONT_SIZE_XL: CssVar = CssVar::new("--lsx-text-font-size-xl");
pub const TEXT_LETTER_SPACING_XL: CssVar = CssVar::new("--lsx-text-letter-spacing-xl");
pub const TEXT_LINE_HEIGHT_XL: CssVar = CssVar::new("--lsx-text-line-height-xl");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextSize {
    pub font_weight: &'static str,
    pub font_size: &'static str, // in rem
    pub letter_spacing: &'static str,
    pub line_height: &'static str,
}

impl TextSize {
    pub const fn new(
        font_weight: &'static str,
        font_size: &'static str,
        letter_spacing: &'static str,
        line_height: &'static str,
    ) -> Self {
        Self {
            font_weight,
            font_size,
            letter_spacing,
            line_height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextDefaults {
    pub font_family: &'static str,
    pub xs: TextSize,
    pub sm: TextSize,
    pub md: TextSize,
    pub lg: TextSize,
    pub xl: TextSize,
}

impl TextDefaults {
    pub const fn new(
        font_family: &'static str,
        xs: TextSize,
        sm: TextSize,
        md: TextSize,
        lg: TextSize,
        xl: TextSize,
    ) -> Self {
        Self {
            font_family,
            xs,
            sm,
            md,
            lg,
            xl,
        }
    }

    pub fn xs_sx() -> Sx {
        sx().font_family(TEXT_FONT_FAMILY.value())
            .font_size(TEXT_FONT_SIZE_XS.value())
            .font_weight(TEXT_FONT_WEIGHT_XS.value())
            .letter_spacing(TEXT_LETTER_SPACING_XS.value())
            .line_height(TEXT_LINE_HEIGHT_XS.value())
    }

    pub fn sm_sx() -> Sx {
        sx().font_family(TEXT_FONT_FAMILY.value())
            .font_size(TEXT_FONT_SIZE_SM.value())
            .font_weight(TEXT_FONT_WEIGHT_SM.value())
            .letter_spacing(TEXT_LETTER_SPACING_SM.value())
            .line_height(TEXT_LINE_HEIGHT_SM.value())
    }

    pub fn md_sx() -> Sx {
        sx().font_family(TEXT_FONT_FAMILY.value())
            .font_size(TEXT_FONT_SIZE_MD.value())
            .font_weight(TEXT_FONT_WEIGHT_MD.value())
            .letter_spacing(TEXT_LETTER_SPACING_MD.value())
            .line_height(TEXT_LINE_HEIGHT_MD.value())
    }

    pub fn lg_sx() -> Sx {
        sx().font_family(TEXT_FONT_FAMILY.value())
            .font_size(TEXT_FONT_SIZE_LG.value())
            .font_weight(TEXT_FONT_WEIGHT_LG.value())
            .letter_spacing(TEXT_LETTER_SPACING_LG.value())
            .line_height(TEXT_LINE_HEIGHT_LG.value())
    }

    pub fn xl_sx() -> Sx {
        sx().font_family(TEXT_FONT_FAMILY.value())
            .font_size(TEXT_FONT_SIZE_XL.value())
            .font_weight(TEXT_FONT_WEIGHT_XL.value())
            .letter_spacing(TEXT_LETTER_SPACING_XL.value())
            .line_height(TEXT_LINE_HEIGHT_XL.value())
    }
}
