use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};

use crate::theme::CssVar;

pub const H1_FONT_FAMILY: CssVar = CssVar::new("--lsx-h1-font-family");
pub const H1_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h1-font-weight");
pub const H1_FONT_SIZE: CssVar = CssVar::new("--lsx-h1-font-size");
pub const H1_LETTER_SPACING: CssVar = CssVar::new("--lsx-h1-letter-spacing");
pub const H1_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h1-line-height");

pub const H2_FONT_FAMILY: CssVar = CssVar::new("--lsx-h2-font-family");
pub const H2_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h2-font-weight");
pub const H2_FONT_SIZE: CssVar = CssVar::new("--lsx-h2-font-size");
pub const H2_LETTER_SPACING: CssVar = CssVar::new("--lsx-h2-letter-spacing");
pub const H2_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h2-line-height");

pub const H3_FONT_FAMILY: CssVar = CssVar::new("--lsx-h3-font-family");
pub const H3_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h3-font-weight");
pub const H3_FONT_SIZE: CssVar = CssVar::new("--lsx-h3-font-size");
pub const H3_LETTER_SPACING: CssVar = CssVar::new("--lsx-h3-letter-spacing");
pub const H3_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h3-line-height");

pub const H4_FONT_FAMILY: CssVar = CssVar::new("--lsx-h4-font-family");
pub const H4_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h4-font-weight");
pub const H4_FONT_SIZE: CssVar = CssVar::new("--lsx-h4-font-size");
pub const H4_LETTER_SPACING: CssVar = CssVar::new("--lsx-h4-letter-spacing");
pub const H4_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h4-line-height");

pub const H5_FONT_FAMILY: CssVar = CssVar::new("--lsx-h5-font-family");
pub const H5_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h5-font-weight");
pub const H5_FONT_SIZE: CssVar = CssVar::new("--lsx-h5-font-size");
pub const H5_LETTER_SPACING: CssVar = CssVar::new("--lsx-h5-letter-spacing");
pub const H5_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h5-line-height");

pub const H6_FONT_FAMILY: CssVar = CssVar::new("--lsx-h6-font-family");
pub const H6_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h6-font-weight");
pub const H6_FONT_SIZE: CssVar = CssVar::new("--lsx-h6-font-size");
pub const H6_LETTER_SPACING: CssVar = CssVar::new("--lsx-h6-letter-spacing");
pub const H6_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h6-line-height");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleLevel {
    pub font_family: &'static str,
    pub font_weight: &'static str,
    pub font_size: &'static str, // in rem
    pub letter_spacing: &'static str,
    pub line_height: &'static str,
}

impl TitleLevel {
    pub const fn new(
        font_family: &'static str,
        font_weight: &'static str,
        font_size: &'static str,
        letter_spacing: &'static str,
        line_height: &'static str,
    ) -> Self {
        Self {
            font_family,
            font_weight,
            font_size,
            letter_spacing,
            line_height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleDefaults {
    pub h1: TitleLevel,
    pub h2: TitleLevel,
    pub h3: TitleLevel,
    pub h4: TitleLevel,
    pub h5: TitleLevel,
    pub h6: TitleLevel,
}

impl TitleDefaults {
    pub const fn new(
        h1: TitleLevel,
        h2: TitleLevel,
        h3: TitleLevel,
        h4: TitleLevel,
        h5: TitleLevel,
        h6: TitleLevel,
    ) -> Self {
        Self {
            h1,
            h2,
            h3,
            h4,
            h5,
            h6,
        }
    }

    pub fn h1_sx() -> Sx {
        sx().font_family(H1_FONT_FAMILY.value())
            .font_size(H1_FONT_SIZE.value())
            .font_weight(H1_FONT_WEIGHT.value())
            .letter_spacing(H1_LETTER_SPACING.value())
            .line_height(H1_LINE_HEIGHT.value())
    }

    pub fn h2_sx() -> Sx {
        sx().font_family(H2_FONT_FAMILY.value())
            .font_size(H2_FONT_SIZE.value())
            .font_weight(H2_FONT_WEIGHT.value())
            .letter_spacing(H2_LETTER_SPACING.value())
            .line_height(H2_LINE_HEIGHT.value())
    }

    pub fn h3_sx() -> Sx {
        sx().font_family(H3_FONT_FAMILY.value())
            .font_size(H3_FONT_SIZE.value())
            .font_weight(H3_FONT_WEIGHT.value())
            .letter_spacing(H3_LETTER_SPACING.value())
            .line_height(H3_LINE_HEIGHT.value())
    }

    pub fn h4_sx() -> Sx {
        sx().font_family(H4_FONT_FAMILY.value())
            .font_size(H4_FONT_SIZE.value())
            .font_weight(H4_FONT_WEIGHT.value())
            .letter_spacing(H4_LETTER_SPACING.value())
            .line_height(H4_LINE_HEIGHT.value())
    }

    pub fn h5_sx() -> Sx {
        sx().font_family(H5_FONT_FAMILY.value())
            .font_size(H5_FONT_SIZE.value())
            .font_weight(H5_FONT_WEIGHT.value())
            .letter_spacing(H5_LETTER_SPACING.value())
            .line_height(H5_LINE_HEIGHT.value())
    }

    pub fn h6_sx() -> Sx {
        sx().font_family(H6_FONT_FAMILY.value())
            .font_size(H6_FONT_SIZE.value())
            .font_weight(H6_FONT_WEIGHT.value())
            .letter_spacing(H6_LETTER_SPACING.value())
            .line_height(H6_LINE_HEIGHT.value())
    }
}

impl ToCssDeclarations for TitleDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let levels = [
            (
                &self.h1,
                H1_FONT_FAMILY,
                H1_FONT_WEIGHT,
                H1_FONT_SIZE,
                H1_LETTER_SPACING,
                H1_LINE_HEIGHT,
            ),
            (
                &self.h2,
                H2_FONT_FAMILY,
                H2_FONT_WEIGHT,
                H2_FONT_SIZE,
                H2_LETTER_SPACING,
                H2_LINE_HEIGHT,
            ),
            (
                &self.h3,
                H3_FONT_FAMILY,
                H3_FONT_WEIGHT,
                H3_FONT_SIZE,
                H3_LETTER_SPACING,
                H3_LINE_HEIGHT,
            ),
            (
                &self.h4,
                H4_FONT_FAMILY,
                H4_FONT_WEIGHT,
                H4_FONT_SIZE,
                H4_LETTER_SPACING,
                H4_LINE_HEIGHT,
            ),
            (
                &self.h5,
                H5_FONT_FAMILY,
                H5_FONT_WEIGHT,
                H5_FONT_SIZE,
                H5_LETTER_SPACING,
                H5_LINE_HEIGHT,
            ),
            (
                &self.h6,
                H6_FONT_FAMILY,
                H6_FONT_WEIGHT,
                H6_FONT_SIZE,
                H6_LETTER_SPACING,
                H6_LINE_HEIGHT,
            ),
        ];

        let mut declarations = Vec::new();
        for (level, family_var, weight_var, size_var, spacing_var, height_var) in levels {
            declarations.push(family_var.declare(level.font_family));
            declarations.push(weight_var.declare(level.font_weight));
            declarations.push(size_var.declare(level.font_size));
            declarations.push(spacing_var.declare(level.letter_spacing));
            declarations.push(height_var.declare(level.line_height));
        }
        declarations
    }
}
