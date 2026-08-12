use crate::theme::{Color, ColorCss, ColorValue, NamedColorCss};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CssColorValue(pub(crate) ColorValue);

impl CssColorValue {
    pub(crate) fn value(self) -> String {
        match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCss::PRIMARY.value(shade),
                Color::Secondary => ColorCss::SECONDARY.value(shade),
                Color::Error => ColorCss::ERROR.value(shade),
                Color::Warning => ColorCss::WARNING.value(shade),
                Color::Info => ColorCss::INFO.value(shade),
                Color::Success => ColorCss::SUCCESS.value(shade),
                Color::Grey => ColorCss::GREY.value(shade),
                Color::Black => NamedColorCss::BLACK.value(),
                Color::White => NamedColorCss::WHITE.value(),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCss::PRIMARY_CONTRAST.value(shade),
                Color::Secondary => ColorCss::SECONDARY_CONTRAST.value(shade),
                Color::Error => ColorCss::ERROR_CONTRAST.value(shade),
                Color::Warning => ColorCss::WARNING_CONTRAST.value(shade),
                Color::Info => ColorCss::INFO_CONTRAST.value(shade),
                Color::Success => ColorCss::SUCCESS_CONTRAST.value(shade),
                Color::Grey => ColorCss::GREY_CONTRAST.value(shade),
                Color::Black => NamedColorCss::BLACK.value(),
                Color::White => NamedColorCss::WHITE.value(),
            },
        }
    }

    pub(crate) fn var_name(self) -> String {
        match self.0 {
            ColorValue::Shade(color, shade) => match color {
                Color::Primary => ColorCss::PRIMARY.name(shade),
                Color::Secondary => ColorCss::SECONDARY.name(shade),
                Color::Error => ColorCss::ERROR.name(shade),
                Color::Warning => ColorCss::WARNING.name(shade),
                Color::Info => ColorCss::INFO.name(shade),
                Color::Success => ColorCss::SUCCESS.name(shade),
                Color::Grey => ColorCss::GREY.name(shade),
                Color::Black => NamedColorCss::BLACK.name().to_string(),
                Color::White => NamedColorCss::WHITE.name().to_string(),
            },
            ColorValue::Contrast(color, shade) => match color {
                Color::Primary => ColorCss::PRIMARY_CONTRAST.name(shade),
                Color::Secondary => ColorCss::SECONDARY_CONTRAST.name(shade),
                Color::Error => ColorCss::ERROR_CONTRAST.name(shade),
                Color::Warning => ColorCss::WARNING_CONTRAST.name(shade),
                Color::Info => ColorCss::INFO_CONTRAST.name(shade),
                Color::Success => ColorCss::SUCCESS_CONTRAST.name(shade),
                Color::Grey => ColorCss::GREY_CONTRAST.name(shade),
                Color::Black => NamedColorCss::BLACK.name().to_string(),
                Color::White => NamedColorCss::WHITE.name().to_string(),
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
