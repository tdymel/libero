use crate::sx::{ColorRole, ThemeAwareValue};
use crate::tokens::{Color, ColorShade, ColorValue, HOVER_TINT_SHADE, SELECTED_TINT_SHADE};

// A bare color name carries no shade, so it takes this over the sx
// pipeline's generic default.
const DEFAULT_SHADE: ColorShade = ColorShade::S6;

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

/// `base` as a text colour: the first step of its ramp that reads on the
/// surface. A literal colour has no ramp and comes back as itself.
///
/// This is the same resolution `sx`'s `color()` does for a colour written
/// straight into a rule; components go through here because their colour
/// travels to the rule through a custom property, which `sx` cannot type.
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

    Some(ColorValue::Fill(*color, selected).value())
}

/// A specific shade of `base`'s color, for a variant that pins one rather
/// than stepping from the base. `None` for a literal base, which has no ramp.
pub(crate) fn shade_color(base: &ThemeAwareValue, shade: ColorShade) -> Option<String> {
    let ThemeAwareValue::ColorValue(ColorValue::Shade(color, _)) = base else {
        return None;
    };

    Some(ColorValue::Fill(*color, shade).value())
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

/// `base`'s label over the hover and selected tints: the text role, darkened
/// where the theme's tint needs it. `None` for a literal base, which has no tint.
pub(crate) fn on_tint_color(base: &ThemeAwareValue) -> Option<String> {
    let ThemeAwareValue::ColorValue(shade @ ColorValue::Shade(..)) = base else {
        return None;
    };
    let name = shade.on_tint_name()?;

    Some(format!("var({name}, {})", text_color(base)?))
}

/// The foreground of [`hover_color`]'s filled step: a darker fill can need the
/// other end (`muted`'s black label read 2.59:1 on its hover). `None` for a
/// literal base.
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

    /// On the fill ramp, not the brand one: the resting background is
    /// `fill-6`, so a hover that named `primary.7` would be a step of the
    /// wrong ramp and could land on the same colour.
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
