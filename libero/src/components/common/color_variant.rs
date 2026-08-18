use crate::sx::ThemeAwareValue;
use crate::tokens::{Color, ColorShade, ColorValue};

// A bare theme color name (e.g. "primary") has no shade of its own, so it's
// resolved to this rather than the sx pipeline's generic default (5).
const DEFAULT_SHADE: ColorShade = ColorShade::S6;

// Outlined/text-like variants tint their hover with a fixed light shade
// instead of darkening, since their resting background is transparent.
const HOVER_TINT_SHADE: ColorShade = ColorShade::S1;

/// Resolves a `color` prop into the base color a filled/outlined/plain
/// variant is built from. Anything but a bare theme color name - an explicit
/// shade/contrast, or a literal like `"red"`/`#123456`/`rgb(...)` - passes
/// through unchanged for the normal sx-to-css pipeline to resolve.
pub(crate) fn base_color(value: Option<&ThemeAwareValue>) -> ThemeAwareValue {
    match value {
        None => ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, DEFAULT_SHADE)),
        Some(ThemeAwareValue::Color(color)) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, DEFAULT_SHADE))
        }
        Some(other) => other.clone(),
    }
}

/// The auto-contrast text color to pair with `base` - only a resolved theme
/// shade has a precomputed contrast var; a literal color has none.
pub(crate) fn contrast_color(base: &ThemeAwareValue) -> Option<ThemeAwareValue> {
    match base {
        ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) => Some(
            ThemeAwareValue::ColorValue(ColorValue::Contrast(*color, *shade)),
        ),
        _ => None,
    }
}

/// The hover color for `base` - a darker shade when the variant already
/// paints `base` as its background, a fixed light tint otherwise. `None` for
/// a literal/raw base, which has no theme shade to derive either from.
pub(crate) fn hover_color(base: &ThemeAwareValue, filled: bool) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) = base else {
        return None;
    };

    let hover = if filled {
        shade.darker()
    } else {
        HOVER_TINT_SHADE
    };

    Some(ColorValue::Shade(*color, hover).value())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_theme_color_gets_the_default_shade() {
        assert_eq!(
            base_color(Some(&ThemeAwareValue::Color(Color::Error))),
            ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Error, DEFAULT_SHADE))
        );
        assert_eq!(
            base_color(None),
            ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, DEFAULT_SHADE))
        );
    }

    #[test]
    fn a_literal_color_passes_through_and_has_no_contrast_or_hover() {
        let literal = ThemeAwareValue::from("#123456");
        assert_eq!(base_color(Some(&literal)), literal);
        assert_eq!(contrast_color(&literal), None);
        assert_eq!(hover_color(&literal, true), None);
        assert_eq!(hover_color(&literal, false), None);
    }

    #[test]
    fn filled_hover_darkens_while_the_rest_tint() {
        let base = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, ColorShade::S6));

        assert_eq!(
            hover_color(&base, true),
            Some(ColorValue::Shade(Color::Primary, ColorShade::S6.darker()).value())
        );
        assert_eq!(
            hover_color(&base, false),
            Some(ColorValue::Shade(Color::Primary, HOVER_TINT_SHADE).value())
        );
    }
}
