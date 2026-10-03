use crate::tokens::{
    Color, ColorShade, ColorValue, CssVar, HexColor, NamedColorCss, NegativeSize, Size, SizeCss,
};

use super::{BreakpointValue, ColorRole};
use crate::utils::warn;

/// What a caller writes for [`NamedColorCss::TEXT_DIMMED`].
const DIMMED_TEXT_TOKEN: &str = "text-dimmed";

/// An `sx` value: a size word (`"md"`), a palette colour (`"primary"`, `"primary.7"`),
/// a var, a number, or plain CSS text. Built by `From`, never by hand.
///
/// CSS text reaches the stylesheet unescaped: never build it from user text.
///
/// ```
/// # use libero::sx::ThemeAwareValue;
/// let value: ThemeAwareValue = "primary.7".into();
/// ```
///
/// Docs: <https://libero-ui.dev/about/styling>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThemeAwareValue {
    /// Plain CSS text, written as is.
    String(String),
    /// A Rust number, written as is.
    Number(String),
    /// A size word, read off the property's scale (`"md"`).
    Size(Size),
    /// A size read off its scale in the negative direction, e.g. `"-md"`.
    NegativeSize(Size),
    /// A palette colour at its default shade (`"primary"`).
    Color(Color),
    /// A palette colour at a shade (`"primary.7"`) or its contrast (`"primary-contrast"`).
    ColorValue(ColorValue),
    /// A custom property, written as `var(..)`.
    CssVar(CssVar),
    /// One value per breakpoint, from [`bp()`](crate::sx::bp).
    BreakpointValue(BreakpointValue),
    /// A literal colour: its text for CSS, its RGB for contrast. A translucent
    /// `rgba()` stays a `String`.
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

    /// This value in `role`. A literal colour, var or keyword has no ramp and
    /// comes back unchanged.
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

    /// Focus-ring colour on this value as a `background`. `None` when contrast
    /// is unknown (named colours, `hsl()`, vars, gradients): keep the inherited one.
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
    /// The non-owning variants, classified before any allocation. `None` leaves
    /// the two owning ones to the caller.
    fn parse_borrowed(value: &str) -> Option<Self> {
        // A role name, so before the palette.
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

        // Without a shade or `-contrast` this would only repeat `Color::parse`.
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

/// Warns once per spelling on words that silently miss the theme: `gray`/`grey` and their
/// shades (the palette's old name, now `muted`), `white`/`black` (use `surface`/`ink`).
fn warn_misspelled_palette(value: &str) {
    thread_local! {
        static WARNED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    let is_gray =
        |word: &str| word.eq_ignore_ascii_case("gray") || word.eq_ignore_ascii_case("grey");
    let message = if is_gray(value) {
        format!("`{value}` is a CSS keyword, not a palette colour; the palette name is `muted`.")
    } else if let Some((name, shade)) = value.split_once(['.', '-'])
        && is_gray(name)
        && !shade.is_empty()
        && shade.bytes().all(|byte| byte.is_ascii_digit())
    {
        format!("`{value}` is not a colour; the palette name is `muted`, as in `muted.{shade}`.")
    } else if value.eq_ignore_ascii_case("white") {
        format!(
            "`{value}` is a CSS keyword, not a palette colour; it stays white under a dark theme; `surface` follows the scheme."
        )
    } else if value.eq_ignore_ascii_case("black") {
        format!(
            "`{value}` is a CSS keyword, not a palette colour; it stays black under a dark theme; `ink` follows the scheme."
        )
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
        warn(&message);
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
    /// Allocates only for the two owning variants.
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
    fn a_gray_shade_points_at_the_muted_shade_once() {
        crate::utils::take_warnings();
        for _ in 0..2 {
            assert_eq!(
                ThemeAwareValue::from("gray.7"),
                ThemeAwareValue::String("gray.7".to_string())
            );
        }
        let _ = ThemeAwareValue::from("Grey.3".to_string());
        let _ = ThemeAwareValue::from("grey-5");
        let _ = ThemeAwareValue::from("muted.7");
        let warnings = crate::utils::take_warnings();
        assert_eq!(warnings.len(), 3, "{warnings:?}");
        assert!(
            warnings[0].contains("`gray.7` is not a colour") && warnings[0].contains("`muted.7`")
        );
        assert!(warnings[1].contains("`muted.3`"));
        assert!(warnings[2].contains("`grey-5`") && warnings[2].contains("`muted.5`"));
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

    /// Todo 240: a var, not a palette colour, so no role resolution touches it.
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

    /// A muted colour is taken literally: a disabled label must not darken into looking enabled.
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
        // A muted fill still has to carry its foreground.
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
