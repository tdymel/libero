use crate::tokens::{
    Color, ColorShade, ColorValue, CssVar, HexColor, NamedColorCss, Size, SizeCss,
};

use super::BreakpointValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThemeAwareValue {
    String(String),
    Number(String),
    Size(Size),
    Color(Color),
    ColorValue(ColorValue),
    CssVar(CssVar),
    BreakpointValue(BreakpointValue),
    /// A literal color: original text for CSS output, parsed RGB for contrast
    /// lookups. A translucent `rgba()` stays a `String` instead.
    RawColor(String, HexColor),
}

impl ThemeAwareValue {
    /// CSS text. A bare `Size` needs `scale`, and is `None` without one.
    pub(crate) fn resolve(&self, scale: Option<SizeCss>) -> Option<String> {
        match self {
            Self::String(value) | Self::Number(value) => Some(value.clone()),
            Self::CssVar(css_var) => Some(css_var.value()),
            Self::RawColor(raw, _) => Some(raw.clone()),
            Self::ColorValue(color_value) => Some(color_value.value()),
            Self::Color(color) => Some(ColorValue::Shade(*color, ColorShade::DEFAULT).value()),
            Self::Size(size) => scale.map(|scale| scale.value(*size)),
            Self::BreakpointValue(_) => None,
        }
    }

    /// Focus-ring color for this value used as a `background`. `None` when
    /// the contrast can't be determined (named colors, `hsl()`, vars,
    /// gradients) - leave the inherited value alone rather than clearing it.
    pub(crate) fn focus_contrast(&self) -> Option<String> {
        match self {
            Self::Color(color) => Some(ColorValue::Contrast(*color, ColorShade::DEFAULT).value()),
            Self::ColorValue(ColorValue::Shade(color, shade)) => {
                Some(ColorValue::Contrast(*color, *shade).value())
            }
            Self::RawColor(_, hex) => Some(if hex.contrast().rgb() == 0x00_00_00 {
                NamedColorCss::BLACK.value()
            } else {
                NamedColorCss::WHITE.value()
            }),
            _ => None,
        }
    }
}

impl From<String> for ThemeAwareValue {
    fn from(value: String) -> Self {
        if let Some(size) = Size::parse_dynamic(value.as_str()) {
            return Self::Size(size);
        }

        if let Some(color) = Color::parse(value.as_str()) {
            return Self::Color(color);
        }

        if let Some(color) = ColorValue::parse(value.as_str()) {
            return Self::ColorValue(color);
        }

        if let Some(css_var) = CssVar::parse(value.as_str()) {
            return Self::CssVar(css_var);
        }

        if let Some(hex) = HexColor::parse(value.as_str()) {
            return Self::RawColor(value, hex);
        }

        Self::String(value)
    }
}

impl From<&str> for ThemeAwareValue {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<Size> for ThemeAwareValue {
    fn from(value: Size) -> Self {
        Self::Size(value)
    }
}

impl From<Color> for ThemeAwareValue {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

impl From<ColorValue> for ThemeAwareValue {
    fn from(value: ColorValue) -> Self {
        Self::ColorValue(value)
    }
}

impl From<CssVar> for ThemeAwareValue {
    fn from(value: CssVar) -> Self {
        Self::CssVar(value)
    }
}

impl From<BreakpointValue> for ThemeAwareValue {
    fn from(value: BreakpointValue) -> Self {
        Self::BreakpointValue(value)
    }
}

macro_rules! impl_int_into_theme_aware_value {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for ThemeAwareValue {
                fn from(value: $t) -> Self {
                    Self::Number(value.to_string())
                }
            }
        )*
    };
}

impl_int_into_theme_aware_value!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

impl From<f32> for ThemeAwareValue {
    fn from(value: f32) -> Self {
        Self::Number(value.to_string())
    }
}

impl From<f64> for ThemeAwareValue {
    fn from(value: f64) -> Self {
        Self::Number(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::CssVar;

    use super::*;

    #[test]
    fn theme_aware_value_parses_css_vars() {
        assert_eq!(
            ThemeAwareValue::from("--custom-var"),
            ThemeAwareValue::CssVar(CssVar::Owned("--custom-var".to_string()))
        );
        assert_eq!(
            ThemeAwareValue::from("var(--other-var)"),
            ThemeAwareValue::CssVar(CssVar::Owned("--other-var".to_string()))
        );
    }

    #[test]
    fn theme_aware_value_parses_a_bare_color_name_as_color_not_color_value() {
        assert_eq!(
            ThemeAwareValue::from("primary"),
            ThemeAwareValue::Color(crate::tokens::Color::Primary)
        );
        assert_eq!(
            ThemeAwareValue::from(crate::tokens::Color::Secondary),
            ThemeAwareValue::Color(crate::tokens::Color::Secondary)
        );
    }

    #[test]
    fn theme_aware_value_parses_an_explicit_shade_as_color_value() {
        assert_eq!(
            ThemeAwareValue::from("primary.7"),
            ThemeAwareValue::ColorValue(crate::tokens::ColorValue::Shade(
                crate::tokens::Color::Primary,
                crate::tokens::ColorShade::S7
            ))
        );
    }
}
