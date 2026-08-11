use crate::theme::{Color, ColorValue};

use super::css_var::{ColorCssVar, NamedColorCssVar};

impl ColorValue {
    pub(crate) fn css_value(self) -> String {
        match self {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.value(shade),
                Color::Secondary => ColorCssVar::SECONDARY.value(shade),
                Color::Black => NamedColorCssVar::BLACK.value(),
                Color::White => NamedColorCssVar::WHITE.value(),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.value(shade),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.value(shade),
                Color::Black => NamedColorCssVar::BLACK.value(),
                Color::White => NamedColorCssVar::WHITE.value(),
            },
        }
    }

    pub(crate) fn css_var_name(self) -> String {
        match self {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.name(shade),
                Color::Secondary => ColorCssVar::SECONDARY.name(shade),
                Color::Black => NamedColorCssVar::BLACK.name(),
                Color::White => NamedColorCssVar::WHITE.name(),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.name(shade),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.name(shade),
                Color::Black => NamedColorCssVar::BLACK.name(),
                Color::White => NamedColorCssVar::WHITE.name(),
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
        let primary = ColorValue::Shade(Color::Primary, ColorShade::S1).css_value();
        let secondary_contrast = ColorValue::Contrast(Color::Secondary, ColorShade::S5).css_value();
        let primary_var = ColorValue::Shade(Color::Primary, ColorShade::S1).css_var_name();

        assert_eq!(primary, "var(--lsx-primary-1)");
        assert_eq!(secondary_contrast, "var(--lsx-secondary-contrast-5)");
        assert_eq!(primary_var, "--lsx-primary-1");
        assert_eq!(
            ColorValue::Shade(Color::Black, ColorShade::S7).css_value(),
            "var(--lsx-black)"
        );
        assert_eq!(
            ColorValue::Shade(Color::White, ColorShade::S7).css_var_name(),
            "--lsx-white"
        );
    }
}
