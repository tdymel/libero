use crate::sx::ThemeAwareValue;
use crate::tokens::{Color, ColorShade, ColorValue};

// A bare color name carries no shade, so it takes this over the sx
// pipeline's generic default.
const DEFAULT_SHADE: ColorShade = ColorShade::S6;

// Transparent at rest, so these tint on hover instead of darkening.
const HOVER_TINT_SHADE: ColorShade = ColorShade::S1;

// One step past the hover tint, so a selected control still reads as selected
// while the pointer is over it.
const SELECTED_TINT_SHADE: ColorShade = ColorShade::S2;

// Same, for a filled control, whose hover is already one step darker.
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

/// Background for a selected/pressed control: one step past what
/// [`hover_color`] would give, in the same direction, so a selected control
/// still reads as selected under the pointer. `None` for a literal base.
pub(crate) fn selected_color(base: &ThemeAwareValue, filled: bool) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, shade)) = base else {
        return None;
    };

    let selected = if filled {
        (0..SELECTED_DARKER_STEPS).fold(*shade, |shade, _| shade.darker())
    } else {
        SELECTED_TINT_SHADE
    };

    Some(ColorValue::Shade(*color, selected).value())
}

/// A specific shade of `base`'s color, for a variant that pins one rather
/// than stepping from the base. `None` for a literal base, which has no ramp.
pub(crate) fn shade_color(base: &ThemeAwareValue, shade: ColorShade) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, _)) = base else {
        return None;
    };

    Some(ColorValue::Shade(*color, shade).value())
}

/// Black or white, whichever reads on `base`'s color at `shade` - the label
/// for a tinted container. Our ramp bottoms out at a 25% black mix, so a dark
/// tone of the hue itself cannot reach 4.5:1 on its own tint for the lighter
/// palette colors (`warning` peaks at 2.9:1). `None` for a literal base.
pub(crate) fn contrast_shade_color(base: &ThemeAwareValue, shade: ColorShade) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, _)) = base else {
        return None;
    };

    Some(ColorValue::Contrast(*color, shade).value())
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
