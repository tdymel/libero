use crate::{
    common::ConstStr,
    theme::{Color, ColorValue},
};

use super::css_var::ColorCssVar;

impl ColorValue {
    pub(crate) const fn push_var_name<const MAX_SIZE: usize>(
        self,
        css: ConstStr<MAX_SIZE>,
    ) -> ConstStr<MAX_SIZE> {
        match self {
            Self::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.push_name_with_shade(css, shade),
                Color::Secondary => ColorCssVar::SECONDARY.push_name_with_shade(css, shade),
            },
            Self::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.push_name_with_shade(css, shade),
                Color::Secondary => {
                    ColorCssVar::SECONDARY_CONTRAST.push_name_with_shade(css, shade)
                }
            },
        }
    }

    pub(crate) const fn push_var<const MAX_SIZE: usize>(
        self,
        css: ConstStr<MAX_SIZE>,
    ) -> ConstStr<MAX_SIZE> {
        match self {
            Self::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.push_var(css, shade),
                Color::Secondary => ColorCssVar::SECONDARY.push_var(css, shade),
            },
            Self::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.push_var(css, shade),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.push_var(css, shade),
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
            ColorValue::Contrast(Color::Secondary, ColorShade::S5).push_var(ConstStr::new());
        const PRIMARY_VAR: ConstStr =
            ColorValue::Shade(Color::Primary, ColorShade::S1).push_var_name(ConstStr::new());

        assert_eq!(PRIMARY.as_str(), "var(--lsx-primary-1)");
        assert_eq!(
            SECONDARY_CONTRAST.as_str(),
            "var(--lsx-secondary-contrast-5)"
        );
        assert_eq!(PRIMARY_VAR.as_str(), "--lsx-primary-1");
    }
}
