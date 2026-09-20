use super::{Color, ColorShade, ShadeRamp, color::ColorVars};

/// The var infix of the [`ColorValue::Text`] ramp.
pub(crate) const TEXT_INFIX: &str = "text-";
/// The var infix of the [`ColorValue::Fill`] ramp.
pub(crate) const FILL_INFIX: &str = "fill-";
/// The var infix of the text ramp re-based onto the hover and selected tints.
pub(crate) const ON_TINT_INFIX: &str = "on-tint-";

/// The hover tint of an unfilled control, transparent at rest.
pub(crate) const HOVER_TINT_SHADE: ColorShade = ColorShade::S1;
/// One past the hover tint, so a selected control still reads under the pointer.
pub(crate) const SELECTED_TINT_SHADE: ColorShade = ColorShade::S2;

/// A palette shade in a role, each its own `:root` ramp: `Shade` is the brand colour,
/// `Text` re-based to read on the surface, `Fill` to carry a black or white `Contrast`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorValue {
    Shade(Color, ColorShade),
    Contrast(Color, ColorShade),
    Text(Color, ColorShade),
    Fill(Color, ColorShade),
}

impl ColorValue {
    /// The same colour in the text role. Neutral ramps stay put: a grey is picked for how
    /// quiet it looks; `"text-dimmed"` is the legible quiet text.
    pub(crate) fn as_text(self) -> Self {
        match self {
            Self::Shade(color, shade) if color.shade_ramp() == ShadeRamp::Chromatic => {
                Self::Text(color, shade)
            }
            other => other,
        }
    }

    /// The same colour in the fill role.
    pub(crate) fn as_fill(self) -> Self {
        match self {
            Self::Shade(color, shade) => Self::Fill(color, shade),
            other => other,
        }
    }
}

impl ColorValue {
    pub(crate) fn value(self) -> String {
        format!("var({})", self.var_name())
    }

    pub(crate) fn var_name(self) -> String {
        let (color, shade, contrast) = match self {
            Self::Shade(color, shade) => (color, shade, false),
            Self::Contrast(color, shade) => (color, shade, true),
            // Ink and surface have no ramp: both roles are the colour itself.
            Self::Text(color, shade) | Self::Fill(color, shade) => {
                let infix = if matches!(self, Self::Text(..)) {
                    TEXT_INFIX
                } else {
                    FILL_INFIX
                };
                return match color.vars() {
                    ColorVars::Palette(own, _) => own.role_name(infix, shade),
                    ColorVars::Named(own, _) => own.name().to_string(),
                };
            }
        };

        match color.vars() {
            ColorVars::Palette(own, other) => if contrast { other } else { own }.name(shade),
            // One var each, no per-shade variants.
            ColorVars::Named(own, other) => if contrast { other } else { own }.name().to_string(),
        }
    }

    /// The var of this shade's label on a hover or selected tint; `None` but for a palette shade.
    pub(crate) fn on_tint_name(self) -> Option<String> {
        match (self, self.color().vars()) {
            (Self::Shade(_, shade), ColorVars::Palette(own, _)) => {
                Some(own.role_name(ON_TINT_INFIX, shade))
            }
            _ => None,
        }
    }

    pub(crate) fn color(self) -> Color {
        match self {
            Self::Shade(color, _)
            | Self::Contrast(color, _)
            | Self::Text(color, _)
            | Self::Fill(color, _) => color,
        }
    }

    /// `"primary"`, `"primary.7"`, or a contrast of either
    /// (`"primary-contrast.7"`). Ink/surface parse only as bare names.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        if value == "ink" {
            return Some(Self::Shade(Color::Ink, ColorShade::S1));
        }

        if value == "surface" {
            return Some(Self::Shade(Color::Surface, ColorShade::S1));
        }

        let (name, shade) = match value.split_once('.') {
            Some((name, shade)) => (name, Some(shade)),
            None => (value, None),
        };
        let (name, contrast) = match name.strip_suffix("-contrast") {
            Some(name) => (name, true),
            None => (name, false),
        };

        // Name first: `rgba(0, 0, 0, 0.15)` must bail before `ColorShade::parse` warns.
        let color = Color::parse(name)?;
        if !matches!(color.vars(), ColorVars::Palette(..)) {
            return None;
        }
        let shade = ColorShade::parse(shade);

        Some(if contrast {
            Self::Contrast(color, shade)
        } else {
            Self::Shade(color, shade)
        })
    }
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
            ColorValue::Shade(Color::Ink, ColorShade::S7).value(),
            "var(--lsx-ink)"
        );
        assert_eq!(
            ColorValue::Shade(Color::Surface, ColorShade::S7).var_name(),
            "--lsx-surface"
        );
    }

    #[test]
    fn black_and_white_contrast_to_each_other() {
        assert_eq!(
            ColorValue::Contrast(Color::Ink, ColorShade::S5).value(),
            "var(--lsx-surface)"
        );
        assert_eq!(
            ColorValue::Contrast(Color::Surface, ColorShade::S5).value(),
            "var(--lsx-ink)"
        );
    }

    #[test]
    fn parse_covers_bare_shaded_and_contrast_forms() {
        assert_eq!(
            ColorValue::parse("primary"),
            Some(ColorValue::Shade(Color::Primary, ColorShade::S6))
        );
        assert_eq!(
            ColorValue::parse("muted.7"),
            Some(ColorValue::Shade(Color::Muted, ColorShade::S7))
        );
        assert_eq!(
            ColorValue::parse("warning-contrast"),
            Some(ColorValue::Contrast(Color::Warning, ColorShade::S6))
        );
        assert_eq!(
            ColorValue::parse("info-contrast.3"),
            Some(ColorValue::Contrast(Color::Info, ColorShade::S3))
        );
        assert_eq!(
            ColorValue::parse("ink"),
            Some(ColorValue::Shade(Color::Ink, ColorShade::S1))
        );
    }

    /// Raw CSS is rejected on the colour name, before `ColorShade::parse` warns.
    #[test]
    fn parse_rejects_raw_css_without_asserting_on_the_shade() {
        assert_eq!(ColorValue::parse("rgba(255, 255, 255, 0.15)"), None);
        assert_eq!(ColorValue::parse("1.5rem"), None);
    }

    #[test]
    fn parse_rejects_non_palette_values() {
        assert_eq!(ColorValue::parse("black.3"), None);
        assert_eq!(ColorValue::parse("white-contrast"), None);
        assert_eq!(ColorValue::parse("primaryish"), None);
        assert_eq!(ColorValue::parse("#ff0000"), None);
    }
}
