use crate::tokens::{
    Color, ColorShade, ColorValue, CssVar, HexColor, NamedColorCss, NegativeSize, Size, SizeCss,
};

use super::BreakpointValue;
use crate::utils::warn;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThemeAwareValue {
    String(String),
    Number(String),
    Size(Size),
    /// A size read off its scale in the negative direction, e.g. `"-md"`.
    NegativeSize(Size),
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
            Self::NegativeSize(size) => {
                scale.map(|scale| format!("calc(-1 * {})", scale.value(*size)))
            }
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

impl ThemeAwareValue {
    /// The variants that borrow nothing, so a `&str` can be classified before
    /// it is allocated. `None` leaves the two owning variants to the caller.
    fn parse_borrowed(value: &str) -> Option<Self> {
        if let Some(size) = Size::parse_dynamic(value) {
            return Some(Self::Size(size));
        }

        if let Some(size) = value.strip_prefix('-').and_then(Size::parse_dynamic) {
            return Some(Self::NegativeSize(size));
        }

        if let Some(color) = Color::parse(value) {
            return Some(Self::Color(color));
        }

        // Without a shade or the contrast suffix, `ColorValue::parse` reduces
        // to the `Color::parse` above - which it also re-runs internally.
        if (value.contains('.') || value.ends_with("-contrast"))
            && let Some(color) = ColorValue::parse(value)
        {
            return Some(Self::ColorValue(color));
        }

        if let Some(css_var) = CssVar::parse(value) {
            return Some(Self::CssVar(css_var));
        }

        None
    }
}

/// `gray` is a CSS keyword, and the palette name is `grey`: the one spelling
/// that falls through to an off-theme colour without a sound.
fn warn_misspelled_palette(value: &str) {
    if value.eq_ignore_ascii_case("gray") {
        warn(&format!(
            "`{value}` is the CSS keyword, not a palette colour; the palette name is `grey`."
        ));
    }
}

impl From<String> for ThemeAwareValue {
    fn from(value: String) -> Self {
        if let Some(parsed) = Self::parse_borrowed(value.as_str()) {
            return parsed;
        }

        match HexColor::parse(value.as_str()) {
            Some(hex) => Self::RawColor(value, hex),
            None => {
                warn_misspelled_palette(&value);
                Self::String(value)
            }
        }
    }
}

impl From<&str> for ThemeAwareValue {
    /// Allocates only for the two owning variants - a token, color or var
    /// value never reaches a `to_string`.
    fn from(value: &str) -> Self {
        if let Some(parsed) = Self::parse_borrowed(value) {
            return parsed;
        }

        match HexColor::parse(value) {
            Some(hex) => Self::RawColor(value.to_string(), hex),
            None => {
                warn_misspelled_palette(value);
                Self::String(value.to_string())
            }
        }
    }
}

impl From<Size> for ThemeAwareValue {
    fn from(value: Size) -> Self {
        Self::Size(value)
    }
}

impl From<NegativeSize> for ThemeAwareValue {
    fn from(value: NegativeSize) -> Self {
        Self::NegativeSize(value.0)
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
    fn gray_stays_the_css_keyword_but_warns() {
        crate::utils::take_warnings();
        assert_eq!(
            ThemeAwareValue::from("gray"),
            ThemeAwareValue::String("gray".to_string())
        );
        assert_eq!(
            ThemeAwareValue::from("grey"),
            ThemeAwareValue::Color(Color::Grey)
        );
        let warnings = crate::utils::take_warnings();
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("`grey`"));
    }

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
    fn theme_aware_value_parses_a_negated_size_token() {
        assert_eq!(
            ThemeAwareValue::from("-md"),
            ThemeAwareValue::NegativeSize(Size::Md)
        );
        assert_eq!(
            ThemeAwareValue::from("-md").resolve(Some(SizeCss::SPACING)),
            Some("calc(-1 * var(--lsx-spacing-md))".to_string())
        );
    }

    /// Only a size word is negated - a CSS length keeps its own sign.
    #[test]
    fn theme_aware_value_leaves_a_negative_length_alone() {
        assert_eq!(
            ThemeAwareValue::from("-8px"),
            ThemeAwareValue::String("-8px".to_string())
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
