use crate::{
    common::ConstStr,
    theme::{Color, ColorValue},
};

use super::{Stylesheet, css_var::ColorCssVar, sx_to_css::DEFAULT_SX_CSS_CAPACITY};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssColorValue(pub ColorValue);

impl CssColorValue {
    pub(crate) const fn to_const_str(self) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
        match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.push_var(ConstStr::new(), shade),
                Color::Secondary => ColorCssVar::SECONDARY.push_var(ConstStr::new(), shade),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.push_var(ConstStr::new(), shade),
                Color::Secondary => {
                    ColorCssVar::SECONDARY_CONTRAST.push_var(ConstStr::new(), shade)
                }
            },
        }
    }

    pub(crate) const fn to_const_str_var_name(self) -> ConstStr<DEFAULT_SX_CSS_CAPACITY> {
        match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.push_name_with_shade(ConstStr::new(), shade),
                Color::Secondary => {
                    ColorCssVar::SECONDARY.push_name_with_shade(ConstStr::new(), shade)
                }
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => {
                    ColorCssVar::PRIMARY_CONTRAST.push_name_with_shade(ConstStr::new(), shade)
                }
                Color::Secondary => {
                    ColorCssVar::SECONDARY_CONTRAST.push_name_with_shade(ConstStr::new(), shade)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::theme::ColorShade;

    use super::*;

    #[test]
    fn color_value_happy_path() {
        let primary = Stylesheet::new().append(
            CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1))
                .to_const_str()
                .as_str(),
        );

        let secondary_contrast = Stylesheet::new().append(
            CssColorValue(ColorValue::Contrast(Color::Secondary, ColorShade::S5))
                .to_const_str()
                .as_str(),
        );

        let primary_var = Stylesheet::new().append(
            CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1))
                .to_const_str_var_name()
                .as_str(),
        );

        assert_eq!(primary.as_str(), "var(--lsx-primary-1)");
        assert_eq!(
            secondary_contrast.as_str(),
            "var(--lsx-secondary-contrast-5)"
        );
        assert_eq!(primary_var.as_str(), "--lsx-primary-1");
    }
}
