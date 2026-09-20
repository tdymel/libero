use crate::sx::{ColorRole, ThemeAwareValue};
use crate::tokens::{
    Color, ColorShade, ColorValue, HOVER_TINT_SHADE, HexColor, SELECTED_TINT_SHADE,
};

// A bare colour name takes this shade.
const DEFAULT_SHADE: ColorShade = ColorShade::S6;

// A filled control's hover is already one step darker.
const SELECTED_DARKER_STEPS: usize = 2;

/// The base color a filled/outlined/plain variant is built from. Anything but
/// a bare theme color name passes through untouched.
pub(crate) fn base_color(value: Option<&ThemeAwareValue>) -> ThemeAwareValue {
    match value {
        None => ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, DEFAULT_SHADE)),
        Some(ThemeAwareValue::Color(color)) => {
            ThemeAwareValue::ColorValue(ColorValue::Shade(*color, DEFAULT_SHADE))
        }
        Some(other) => other.clone(),
    }
}

/// `base` as a text colour, as `sx`'s `color()` resolves it; for colours that travel
/// through a custom property. A literal comes back as itself.
pub(crate) fn text_color(base: &ThemeAwareValue) -> Option<String> {
    base.in_color_role(ColorRole::Text).resolve(None)
}

/// `base` as a background: the first step of its ramp that carries a black or
/// white foreground. Pairs with [`contrast_color`].
pub(crate) fn fill_color(base: &ThemeAwareValue) -> Option<String> {
    base.in_color_role(ColorRole::Fill).resolve(None)
}

/// Auto-contrast text for `base`. `None` for a literal color, which has no
/// precomputed contrast var.
pub(crate) fn contrast_color(base: &ThemeAwareValue) -> Option<ThemeAwareValue> {
    match base {
        ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) => Some(
            ThemeAwareValue::ColorValue(ColorValue::Contrast(*color, *shade)),
        ),
        _ => None,
    }
}

/// Black or white text for a literal fill, whichever clears 4.5:1. An unparsed colour asks
/// the browser's `contrast-color()`.
pub(crate) fn literal_contrast(base: &ThemeAwareValue) -> Option<String> {
    match base {
        ThemeAwareValue::RawColor(_, fill) => {
            let (black, white) = (HexColor::new(0x00_00_00), HexColor::new(0xFF_FF_FF));
            Some(
                match fill.contrast_ratio(black) >= fill.contrast_ratio(white) {
                    true => black,
                    false => white,
                }
                .to_string(),
            )
        }
        ThemeAwareValue::String(_) | ThemeAwareValue::CssVar(_) => {
            Some(format!("contrast-color({})", base.resolve(None)?))
        }
        _ => None,
    }
}

/// Selected background: one step past [`hover_color`], so it still reads under the pointer.
pub(crate) fn selected_color(base: &ThemeAwareValue, filled: bool) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) = base else {
        return None;
    };

    let selected = if filled {
        (0..SELECTED_DARKER_STEPS).fold(*shade, |shade, _| shade.darker())
    } else {
        SELECTED_TINT_SHADE
    };

    Some(ColorValue::Fill(*color, selected).value())
}

/// A pinned shade of `base`'s colour. `None` for a literal base.
pub(crate) fn shade_color(base: &ThemeAwareValue, shade: ColorShade) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, _)) = base else {
        return None;
    };

    Some(ColorValue::Fill(*color, shade).value())
}

/// Black or white on `base` at `shade`, a tinted container's label: a dark tone of the hue
/// misses 4.5:1 on light colours (`warning` peaks at 2.9:1).
pub(crate) fn contrast_shade_color(base: &ThemeAwareValue, shade: ColorShade) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, _)) = base else {
        return None;
    };

    Some(ColorValue::Contrast(*color, shade).value())
}

/// `base`'s label over the hover and selected tints: the text role, darkened
/// where the theme's tint needs it. `None` for a literal base, which has no tint.
pub(crate) fn on_tint_color(base: &ThemeAwareValue) -> Option<String> {
    let ThemeAwareValue::ColorValue(shade @ ColorValue::Shade(..)) = base else {
        return None;
    };
    let name = shade.on_tint_name()?;

    Some(format!("var({name}, {})", text_color(base)?))
}

/// The foreground on [`hover_color`]'s filled step, which can flip (`muted`: 2.59:1).
pub(crate) fn hover_contrast_color(base: &ThemeAwareValue) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) = base else {
        return None;
    };

    Some(ColorValue::Contrast(*color, shade.darker()).value())
}

/// Hover color for `base`: darker when `base` is already the background, a
/// light tint otherwise. `None` for a literal base, which has no shade scale.
pub(crate) fn hover_color(base: &ThemeAwareValue, filled: bool) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) = base else {
        return None;
    };

    let hover = if filled {
        shade.darker()
    } else {
        HOVER_TINT_SHADE
    };

    Some(ColorValue::Fill(*color, hover).value())
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
    fn a_literal_fill_gets_the_readable_end() {
        let ink = |value: &str| literal_contrast(&ThemeAwareValue::from(value));
        assert_eq!(ink("#ffeb3b").as_deref(), Some("#000000"));
        assert_eq!(ink("#123456").as_deref(), Some("#FFFFFF"));
        // `#777` reads 4.48:1 against white but 4.69:1 against black.
        assert_eq!(ink("#777777").as_deref(), Some("#000000"));
        assert_eq!(
            ink("rebeccapurple").as_deref(),
            Some("contrast-color(rebeccapurple)")
        );
        assert_eq!(ink("error"), None);
    }

    /// On the fill ramp, not the brand one, which could land on the resting colour.
    #[test]
    fn filled_hover_darkens_while_the_rest_tint() {
        let base = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, ColorShade::S6));

        assert_eq!(
            hover_color(&base, true),
            Some(ColorValue::Fill(Color::Primary, ColorShade::S6.darker()).value())
        );
        assert_eq!(
            hover_color(&base, false),
            Some(ColorValue::Fill(Color::Primary, HOVER_TINT_SHADE).value())
        );
    }

    #[test]
    fn the_two_roles_resolve_through_their_own_ramps() {
        let base = ThemeAwareValue::ColorValue(ColorValue::Shade(Color::Primary, ColorShade::S6));

        assert_eq!(
            text_color(&base).as_deref(),
            Some("var(--lsx-primary-text-6)")
        );
        assert_eq!(
            fill_color(&base).as_deref(),
            Some("var(--lsx-primary-fill-6)")
        );

        // A literal has no ramp: both roles are the colour the caller wrote.
        let literal = ThemeAwareValue::from("#123456");
        assert_eq!(text_color(&literal).as_deref(), Some("#123456"));
        assert_eq!(fill_color(&literal).as_deref(), Some("#123456"));
    }
}
