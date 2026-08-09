use crate::{
    common::ConstStr,
    theme::{Color, ColorValue},
};

use super::css_var::CssVar;

impl ColorValue {
    pub(crate) const fn push_var_name(self, css: ConstStr) -> ConstStr {
        match self {
            Self::Shade(color, shade) => match color {
                Color::Primary => CssVar::PRIMARY.push_name_with_shade(css, shade),
                Color::Secondary => CssVar::SECONDARY.push_name_with_shade(css, shade),
            },
            Self::Contrast(color) => match color {
                Color::Primary => CssVar::PRIMARY_CONTRAST.push_name(css),
                Color::Secondary => CssVar::SECONDARY_CONTRAST.push_name(css),
            },
        }
    }

    pub(crate) const fn push_css_var(self, css: ConstStr) -> ConstStr {
        match self {
            Self::Shade(color, shade) => match color {
                Color::Primary => CssVar::PRIMARY.push_value_with_shade(css, shade),
                Color::Secondary => CssVar::SECONDARY.push_value_with_shade(css, shade),
            },
            Self::Contrast(color) => match color {
                Color::Primary => CssVar::PRIMARY_CONTRAST.push_value(css),
                Color::Secondary => CssVar::SECONDARY_CONTRAST.push_value(css),
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
            ColorValue::Shade(Color::Primary, ColorShade::S1).push_css_var(ConstStr::new());
        const SECONDARY_CONTRAST: ConstStr =
            ColorValue::Contrast(Color::Secondary).push_css_var(ConstStr::new());
        const PRIMARY_VAR: ConstStr =
            ColorValue::Shade(Color::Primary, ColorShade::S1).push_var_name(ConstStr::new());

        assert_eq!(PRIMARY.as_str(), "var(--lsx-primary-100)");
        assert_eq!(SECONDARY_CONTRAST.as_str(), "var(--lsx-secondary-contrast)");
        assert_eq!(PRIMARY_VAR.as_str(), "--lsx-primary-100");
    }
}
