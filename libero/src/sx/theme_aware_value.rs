use crate::tokens::{
    Color, ColorShade, ColorValue, CssVar, HexColor, NamedColorCss, NegativeSize, Size, SizeCss,
};

use super::{BreakpointValue, ColorRole};
use crate::utils::warn;

/// What a caller writes for [`NamedColorCss::TEXT_DIMMED`].
const DIMMED_TEXT_TOKEN: &str = "text-dimmed";

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

    /// This value in `role`, for the properties that have one. A literal
    /// colour, a var or a keyword has no ramp to move along and is returned
    /// unchanged - the caller asked for exactly that colour.
    pub(crate) fn in_color_role(&self, role: ColorRole) -> Self {
        let in_role = |value: ColorValue| match role {
            ColorRole::Text => value.as_text(),
            ColorRole::Fill => value.as_fill(),
        };

        match self {
            Self::Color(color) => {
                Self::ColorValue(in_role(ColorValue::Shade(*color, ColorShade::DEFAULT)))
            }
            Self::ColorValue(value) => Self::ColorValue(in_role(*value)),
            other => other.clone(),
        }
    }

    /// Focus-ring color for this value used as a `background`. `None` when
    /// the contrast can't be determined (named colors, `hsl()`, vars,
    /// gradients) - leave the inherited value alone rather than clearing it.
    pub(crate) fn focus_contrast(&self) -> Option<String> {
        match self {
            Self::Color(color) => Some(ColorValue::Contrast(*color, ColorShade::DEFAULT).value()),
            Self::ColorValue(ColorValue::Shade(color, shade) | ColorValue::Fill(color, shade)) => {
                Some(ColorValue::Contrast(*color, *shade).value())
            }
            // A literal: the hex does not flip with the scheme, `--lsx-ink` does.
            Self::RawColor(_, hex) => Some(hex.contrast().to_string()),
            _ => None,
        }
    }
}

impl ThemeAwareValue {
    /// The variants that borrow nothing, so a `&str` can be classified before
    /// it is allocated. `None` leaves the two owning variants to the caller.
    fn parse_borrowed(value: &str) -> Option<Self> {
        // Before the palette: a role name, not a colour name.
        if value == DIMMED_TEXT_TOKEN {
            return Some(Self::CssVar(NamedColorCss::TEXT_DIMMED.var()));
        }

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

/// `gray` and `grey` are both CSS keywords, and the palette name is `muted`:
/// the two spellings that fall through to an off-theme colour without a
/// sound. `grey` named the palette until the ramp was re-based on the
/// surface, so a call site that predates the rename keeps compiling and
/// paints the CSS keyword. Once per spelling, since this runs on every render
/// of every `color: "gray"`.
///
/// `white` and `black` are the same trap in a dark theme: a fixed colour where
/// `surface` or `ink` would follow the scheme.
fn warn_misspelled_palette(value: &str) {
    thread_local! {
        static WARNED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    let advice = if value.eq_ignore_ascii_case("gray") || value.eq_ignore_ascii_case("grey") {
        "the palette name is `muted`"
    } else if value.eq_ignore_ascii_case("white") {
        "it stays white under a dark theme; `surface` follows the scheme"
    } else if value.eq_ignore_ascii_case("black") {
        "it stays black under a dark theme; `ink` follows the scheme"
    } else {
        return;
    };
    if WARNED.with_borrow_mut(|warned| {
        let first = !warned.iter().any(|spelling| spelling == value);
        if first {
            warned.push(value.to_string());
        }
        first
    }) {
        warn(&format!(
            "`{value}` is a CSS keyword, not a palette colour; {advice}."
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
    fn gray_and_grey_stay_css_keywords_but_warn_once_each() {
        crate::utils::take_warnings();
        assert_eq!(
            ThemeAwareValue::from("gray"),
            ThemeAwareValue::String("gray".to_string())
        );
        assert_eq!(
            ThemeAwareValue::from("gray".to_string()),
            ThemeAwareValue::String("gray".to_string())
        );
        assert_eq!(
            ThemeAwareValue::from("grey"),
            ThemeAwareValue::String("grey".to_string())
        );
        assert_eq!(
            ThemeAwareValue::from("muted"),
            ThemeAwareValue::Color(Color::Muted)
        );
        let warnings = crate::utils::take_warnings();
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(warnings.iter().all(|warning| warning.contains("`muted`")));
    }

    #[test]
    fn white_and_black_point_at_surface_and_ink_once_each() {
        crate::utils::take_warnings();
        for _ in 0..2 {
            assert_eq!(
                ThemeAwareValue::from("white"),
                ThemeAwareValue::String("white".to_string())
            );
            assert_eq!(
                ThemeAwareValue::from("black".to_string()),
                ThemeAwareValue::String("black".to_string())
            );
        }
        for silent in ["surface", "ink", "#fff", "#000000"] {
            let _ = ThemeAwareValue::from(silent);
        }
        let warnings = crate::utils::take_warnings();
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(warnings[0].contains("`white`") && warnings[0].contains("`surface`"));
        assert!(warnings[1].contains("`black`") && warnings[1].contains("`ink`"));
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

    /// Todo 240: the one name for quieter text. It is a var, not a palette
    /// colour, so no role resolution touches it afterwards.
    #[test]
    fn theme_aware_value_parses_the_dimmed_text_token() {
        assert_eq!(
            ThemeAwareValue::from("text-dimmed"),
            ThemeAwareValue::CssVar(crate::tokens::NamedColorCss::TEXT_DIMMED.var())
        );
        assert_eq!(
            ThemeAwareValue::from("text-dimmed")
                .resolve(None)
                .as_deref(),
            Some("var(--lsx-text-dimmed)")
        );
    }

    /// A muted colour is picked for how quiet it looks, so it is taken literally -
    /// a disabled label and a chevron must not darken into looking enabled.
    #[test]
    fn the_text_role_moves_an_accent_and_leaves_a_muted_one_alone() {
        use crate::tokens::{Color, ColorShade, ColorValue};

        assert_eq!(
            ThemeAwareValue::from("primary").in_color_role(ColorRole::Text),
            ThemeAwareValue::ColorValue(ColorValue::Text(Color::Primary, ColorShade::S6))
        );
        assert_eq!(
            ThemeAwareValue::from("muted.6").in_color_role(ColorRole::Text),
            ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Muted, ColorShade::S6))
        );
        // The fill role has no such exception: a muted fill still has to
        // carry its foreground.
        assert_eq!(
            ThemeAwareValue::from("muted.6").in_color_role(ColorRole::Fill),
            ThemeAwareValue::ColorValue(ColorValue::Fill(Color::Muted, ColorShade::S6))
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
