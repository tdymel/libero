use super::CssVar;

// H1
pub const H1_FONT_FAMILY: CssVar = CssVar::new("--lsx-h1-font-family");
pub const H1_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h1-font-weight");
pub const H1_FONT_SIZE: CssVar = CssVar::new("--lsx-h1-font-size");
pub const H1_LETTER_SPACING: CssVar = CssVar::new("--lsx-h1-letter-spacing");
pub const H1_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h1-line-height");

// H2
pub const H2_FONT_FAMILY: CssVar = CssVar::new("--lsx-h2-font-family");
pub const H2_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h2-font-weight");
pub const H2_FONT_SIZE: CssVar = CssVar::new("--lsx-h2-font-size");
pub const H2_LETTER_SPACING: CssVar = CssVar::new("--lsx-h2-letter-spacing");
pub const H2_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h2-line-height");

// H3
pub const H3_FONT_FAMILY: CssVar = CssVar::new("--lsx-h3-font-family");
pub const H3_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h3-font-weight");
pub const H3_FONT_SIZE: CssVar = CssVar::new("--lsx-h3-font-size");
pub const H3_LETTER_SPACING: CssVar = CssVar::new("--lsx-h3-letter-spacing");
pub const H3_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h3-line-height");

// H4
pub const H4_FONT_FAMILY: CssVar = CssVar::new("--lsx-h4-font-family");
pub const H4_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h4-font-weight");
pub const H4_FONT_SIZE: CssVar = CssVar::new("--lsx-h4-font-size");
pub const H4_LETTER_SPACING: CssVar = CssVar::new("--lsx-h4-letter-spacing");
pub const H4_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h4-line-height");

// H5
pub const H5_FONT_FAMILY: CssVar = CssVar::new("--lsx-h5-font-family");
pub const H5_FONT_WEIGHT: CssVar = CssVar::new("--lsx-h5-font-weight");
pub const H5_FONT_SIZE: CssVar = CssVar::new("--lsx-h5-font-size");
pub const H5_LETTER_SPACING: CssVar = CssVar::new("--lsx-h5-letter-spacing");
pub const H5_LINE_HEIGHT: CssVar = CssVar::new("--lsx-h5-line-height");

// H6
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
}
