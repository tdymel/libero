use crate::theme::{Color, ColorValue};

use super::css_var::{ColorCssVar, NamedColorCssVar};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CssColorValue(pub(crate) ColorValue);

impl CssColorValue {
    pub(crate) fn value(self) -> String {
        match self.0 {
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

    pub(crate) fn var_name(self) -> String {
        match self.0 {
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
        let primary = CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1)).value();
        let secondary_contrast =
            CssColorValue(ColorValue::Contrast(Color::Secondary, ColorShade::S5)).value();
        let primary_var =
            CssColorValue(ColorValue::Shade(Color::Primary, ColorShade::S1)).var_name();
        let error = CssColorValue(ColorValue::Shade(Color::Error, ColorShade::S3)).value();
        let success_contrast =
            CssColorValue(ColorValue::Contrast(Color::Success, ColorShade::S4)).var_name();

        assert_eq!(primary, "var(--lsx-primary-1)");
        assert_eq!(secondary_contrast, "var(--lsx-secondary-contrast-5)");
        assert_eq!(primary_var, "--lsx-primary-1");
        assert_eq!(error, "var(--lsx-error-3)");
        assert_eq!(success_contrast, "--lsx-success-contrast-4");
        assert_eq!(
            CssColorValue(ColorValue::Shade(Color::Black, ColorShade::S7)).value(),
            "var(--lsx-black)"
        );
        assert_eq!(
            CssColorValue(ColorValue::Shade(Color::White, ColorShade::S7)).var_name(),
            "--lsx-white"
        );
    }
}
