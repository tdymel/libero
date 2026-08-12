use super::{Color, ColorShade};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorValue {
    Shade(Color, ColorShade),
    Contrast(Color, ColorShade),
}

impl ColorValue {
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
