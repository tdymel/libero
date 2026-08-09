use crate::{
    common::ConstStr,
    theme::{Color, ColorValue},
};

use super::css_var::ColorCssVar;

impl ColorValue {
    pub(crate) const fn push_var_name(self, css: ConstStr) -> ConstStr {
        match self {
            Self::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.push_name_with_shade(css, shade),
                Color::Secondary => ColorCssVar::SECONDARY.push_name_with_shade(css, shade),
            },
            Self::Contrast(color) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.push_name(css),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.push_name(css),
            },
        }
    }

    pub(crate) const fn push_var(self, css: ConstStr) -> ConstStr {
        match self {
            Self::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.push_var(css, shade),
                Color::Secondary => ColorCssVar::SECONDARY.push_var(css, shade),
            },
            Self::Contrast(color) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.push_css_var(css),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.push_css_var(css),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{common::ConstStr, theme::ColorShade};

    use super::*;

    #[test]
    fn color_value_happy_path() {
        const PRIMARY: ConstStr =
            ColorValue::Shade(Color::Primary, ColorShade::S1).push_var(ConstStr::new());
        const SECONDARY_CONTRAST: ConstStr =
            ColorValue::Contrast(Color::Secondary).push_var(ConstStr::new());
        const PRIMARY_VAR: ConstStr =
            ColorValue::Shade(Color::Primary, ColorShade::S1).push_var_name(ConstStr::new());

        assert_eq!(PRIMARY.as_str(), "var(--lsx-primary-100)");
        assert_eq!(SECONDARY_CONTRAST.as_str(), "var(--lsx-secondary-contrast)");
        assert_eq!(PRIMARY_VAR.as_str(), "--lsx-primary-100");
    }
}
