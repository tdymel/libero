use crate::theme::{Color, ColorValue};

use super::{Stylesheet, css_var::ColorCssVar};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssColorValue(pub ColorValue);

impl CssColorValue {
    pub(crate) const fn extend_var_name(self, css: &mut Stylesheet) {
        *css = match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => Stylesheet::from_const_str(
                    ColorCssVar::PRIMARY.push_name_with_shade(css.into_const_str(), shade),
                ),
                Color::Secondary => Stylesheet::from_const_str(
                    ColorCssVar::SECONDARY.push_name_with_shade(css.into_const_str(), shade),
                ),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => Stylesheet::from_const_str(
                    ColorCssVar::PRIMARY_CONTRAST.push_name_with_shade(css.into_const_str(), shade),
                ),
                Color::Secondary => Stylesheet::from_const_str(
                    ColorCssVar::SECONDARY_CONTRAST
                        .push_name_with_shade(css.into_const_str(), shade),
                ),
            },
        };
    }

    pub(crate) const fn extend_var(self, css: &mut Stylesheet) {
        *css = match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => Stylesheet::from_const_str(
                    ColorCssVar::PRIMARY.push_var(css.into_const_str(), shade),
                ),
                Color::Secondary => Stylesheet::from_const_str(
                    ColorCssVar::SECONDARY.push_var(css.into_const_str(), shade),
                ),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => Stylesheet::from_const_str(
                    ColorCssVar::PRIMARY_CONTRAST.push_var(css.into_const_str(), shade),
                ),
                Color::Secondary => Stylesheet::from_const_str(
                    ColorCssVar::SECONDARY_CONTRAST.push_var(css.into_const_str(), shade),
                ),
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::theme::ColorShade;

    use super::*;

    #[test]
    fn color_value_happy_path() {
        let mut primary = Stylesheet::new();
        CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1)).extend_var(&mut primary);

        let mut secondary_contrast = Stylesheet::new();
        CssColorValue(ColorValue::Contrast(Color::Secondary, ColorShade::S5))
            .extend_var(&mut secondary_contrast);

        let mut primary_var = Stylesheet::new();
        CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1))
            .extend_var_name(&mut primary_var);

        assert_eq!(primary.as_str(), "var(--lsx-primary-1)");
        assert_eq!(
            secondary_contrast.as_str(),
            "var(--lsx-secondary-contrast-5)"
        );
        assert_eq!(primary_var.as_str(), "--lsx-primary-1");
    }
}
