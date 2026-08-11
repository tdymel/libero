use crate::theme::{Color, ColorValue};

use super::css_var::{ColorCssVar, NamedColorCssVar};

impl ColorValue {
    pub(crate) fn css_value(self) -> String {
        match self {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY.value(shade),
                Color::Secondary => ColorCssVar::SECONDARY.value(shade),
                Color::Error => ColorCssVar::ERROR.value(shade),
                Color::Warning => ColorCssVar::WARNING.value(shade),
                Color::Info => ColorCssVar::INFO.value(shade),
                Color::Success => ColorCssVar::SUCCESS.value(shade),
                Color::Grey => ColorCssVar::GREY.value(shade),
                Color::Black => NamedColorCssVar::BLACK.value(),
                Color::White => NamedColorCssVar::WHITE.value(),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.value(shade),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.value(shade),
                Color::Error => ColorCssVar::ERROR_CONTRAST.value(shade),
                Color::Warning => ColorCssVar::WARNING_CONTRAST.value(shade),
                Color::Info => ColorCssVar::INFO_CONTRAST.value(shade),
                Color::Success => ColorCssVar::SUCCESS_CONTRAST.value(shade),
                Color::Grey => ColorCssVar::GREY_CONTRAST.value(shade),
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
                Color::Error => ColorCssVar::ERROR.name(shade),
                Color::Warning => ColorCssVar::WARNING.name(shade),
                Color::Info => ColorCssVar::INFO.name(shade),
                Color::Success => ColorCssVar::SUCCESS.name(shade),
                Color::Grey => ColorCssVar::GREY.name(shade),
                Color::Black => NamedColorCssVar::BLACK.name(),
                Color::White => NamedColorCssVar::WHITE.name(),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCssVar::PRIMARY_CONTRAST.name(shade),
                Color::Secondary => ColorCssVar::SECONDARY_CONTRAST.name(shade),
                Color::Error => ColorCssVar::ERROR_CONTRAST.name(shade),
                Color::Warning => ColorCssVar::WARNING_CONTRAST.name(shade),
                Color::Info => ColorCssVar::INFO_CONTRAST.name(shade),
                Color::Success => ColorCssVar::SUCCESS_CONTRAST.name(shade),
                Color::Grey => ColorCssVar::GREY_CONTRAST.name(shade),
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
        let error = ColorValue::Shade(Color::Error, ColorShade::S3).css_value();
        let success_contrast = ColorValue::Contrast(Color::Success, ColorShade::S4).css_var_name();

        assert_eq!(primary, "var(--lsx-primary-1)");
        assert_eq!(secondary_contrast, "var(--lsx-secondary-contrast-5)");
        assert_eq!(primary_var, "--lsx-primary-1");
        assert_eq!(error, "var(--lsx-error-3)");
        assert_eq!(success_contrast, "--lsx-success-contrast-4");
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
