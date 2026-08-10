use crate::theme::{Color, ColorValue};

use super::css_var::ColorCssVar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssColorValue(pub ColorValue);

impl CssColorValue {
    pub(crate) fn to_string(self) -> String {
        match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.to_string(shade),
                Color::Secondary => ColorCssVar::SECONDARY.to_string(shade),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.to_string(shade),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.to_string(shade),
            },
        }
    }

    pub(crate) fn to_string_var_name(self) -> String {
        match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.to_string_name(shade),
                Color::Secondary => ColorCssVar::SECONDARY.to_string_name(shade),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.to_string_name(shade),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.to_string_name(shade),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{css::Stylesheet, theme::ColorShade};

    use super::*;

    #[test]
    fn color_value_happy_path() {
        let primary = Stylesheet::new()
            .append(CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1)).to_string());

        let secondary_contrast = Stylesheet::new().append(
            CssColorValue(ColorValue::Contrast(Color::Secondary, ColorShade::S5)).to_string(),
        );

        let primary_var = Stylesheet::new().append(
            CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1)).to_string_var_name(),
        );

        assert_eq!(primary.as_str(), "var(--lsx-primary-1)");
        assert_eq!(
            secondary_contrast.as_str(),
            "var(--lsx-secondary-contrast-5)"
        );
        assert_eq!(primary_var.as_str(), "--lsx-primary-1");
    }
}
