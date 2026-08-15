use super::{Color, ColorCss, ColorShade, NamedColorCss};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorValue {
    Shade(Color, ColorShade),
    Contrast(Color, ColorShade),
}

impl ColorValue {
    pub(crate) fn value(self) -> String {
        match self {
            Self::Shade(color, shade) => match color {
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
            Self::Contrast(color, shade) => match color {
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
        match self {
            Self::Shade(color, shade) => match color {
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
            Self::Contrast(color, shade) => match color {
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

    pub(crate) fn parse(value: &str) -> Option<Self> {
        if value == "black" {
            return Some(Self::Shade(Color::Black, ColorShade::S1));
        }

        if value == "white" {
            return Some(Self::Shade(Color::White, ColorShade::S1));
        }

        parse_palette(value, "primary-contrast", Color::Primary, true)
            .or_else(|| parse_palette(value, "secondary-contrast", Color::Secondary, true))
            .or_else(|| parse_palette(value, "error-contrast", Color::Error, true))
            .or_else(|| parse_palette(value, "warning-contrast", Color::Warning, true))
            .or_else(|| parse_palette(value, "info-contrast", Color::Info, true))
            .or_else(|| parse_palette(value, "success-contrast", Color::Success, true))
            .or_else(|| parse_palette(value, "grey-contrast", Color::Grey, true))
            .or_else(|| parse_palette(value, "primary", Color::Primary, false))
            .or_else(|| parse_palette(value, "secondary", Color::Secondary, false))
            .or_else(|| parse_palette(value, "error", Color::Error, false))
            .or_else(|| parse_palette(value, "warning", Color::Warning, false))
            .or_else(|| parse_palette(value, "info", Color::Info, false))
            .or_else(|| parse_palette(value, "success", Color::Success, false))
            .or_else(|| parse_palette(value, "grey", Color::Grey, false))
    }
}

fn parse_palette(value: &str, name: &str, color: Color, contrast: bool) -> Option<ColorValue> {
    let suffix = if value == name {
        None
    } else {
        value.strip_prefix(name)?.strip_prefix(".")
    };

    let shade = ColorShade::parse(suffix);
    Some(if contrast {
        ColorValue::Contrast(color, shade)
    } else {
        ColorValue::Shade(color, shade)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_value_happy_path() {
        let primary = ColorValue::Shade(Color::Primary, ColorShade::S1).value();
        let secondary_contrast = ColorValue::Contrast(Color::Secondary, ColorShade::S5).value();
        let primary_var = ColorValue::Shade(Color::Primary, ColorShade::S1).var_name();
        let error = ColorValue::Shade(Color::Error, ColorShade::S3).value();
        let success_contrast = ColorValue::Contrast(Color::Success, ColorShade::S4).var_name();

        assert_eq!(primary, "var(--lsx-primary-1)");
        assert_eq!(secondary_contrast, "var(--lsx-secondary-contrast-5)");
        assert_eq!(primary_var, "--lsx-primary-1");
        assert_eq!(error, "var(--lsx-error-3)");
        assert_eq!(success_contrast, "--lsx-success-contrast-4");
        assert_eq!(
            ColorValue::Shade(Color::Black, ColorShade::S7).value(),
            "var(--lsx-black)"
        );
        assert_eq!(
            ColorValue::Shade(Color::White, ColorShade::S7).var_name(),
            "--lsx-white"
        );
    }
}
