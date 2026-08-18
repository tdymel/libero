use super::{Color, ColorShade, color::ColorVars};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorValue {
    Shade(Color, ColorShade),
    Contrast(Color, ColorShade),
}

impl ColorValue {
    pub(crate) fn value(self) -> String {
        format!("var({})", self.var_name())
    }

    pub(crate) fn var_name(self) -> String {
        let (color, shade, contrast) = match self {
            Self::Shade(color, shade) => (color, shade, false),
            Self::Contrast(color, shade) => (color, shade, true),
        };

        match color.vars() {
            ColorVars::Palette(own, other) => if contrast { other } else { own }.name(shade),
            // Black/white have a single var each, with no per-shade
            // variants, so the shade is irrelevant here.
            ColorVars::Named(own, other) => if contrast { other } else { own }.name().to_string(),
        }
    }

    /// Parses a palette reference: `"primary"`, `"primary.7"`, or the
    /// contrast of either (`"primary-contrast.7"`). Black and white have no
    /// shades of their own, so only their bare names parse.
    pub(crate) fn parse(value: &str) -> Option<Self> {
        if value == "black" {
            return Some(Self::Shade(Color::Black, ColorShade::S1));
        }

        if value == "white" {
            return Some(Self::Shade(Color::White, ColorShade::S1));
        }

        let (name, shade) = match value.split_once('.') {
            Some((name, shade)) => (name, Some(shade)),
            None => (value, None),
        };
        let (name, contrast) = match name.strip_suffix("-contrast") {
            Some(name) => (name, true),
            None => (name, false),
        };

        // Name first: raw CSS like `rgba(0, 0, 0, 0.15)` splits into a
        // nonsense shade, and must bail before `ColorShade::parse` asserts.
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
            ColorValue::Shade(Color::Black, ColorShade::S7).value(),
            "var(--lsx-black)"
        );
        assert_eq!(
            ColorValue::Shade(Color::White, ColorShade::S7).var_name(),
            "--lsx-white"
        );
    }

    #[test]
    fn black_and_white_contrast_to_each_other() {
        assert_eq!(
            ColorValue::Contrast(Color::Black, ColorShade::S5).value(),
            "var(--lsx-white)"
        );
        assert_eq!(
            ColorValue::Contrast(Color::White, ColorShade::S5).value(),
            "var(--lsx-black)"
        );
    }

    #[test]
    fn parse_covers_bare_shaded_and_contrast_forms() {
        assert_eq!(
            ColorValue::parse("primary"),
            Some(ColorValue::Shade(Color::Primary, ColorShade::S6))
        );
        assert_eq!(
            ColorValue::parse("grey.7"),
            Some(ColorValue::Shade(Color::Grey, ColorShade::S7))
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
            ColorValue::parse("black"),
            Some(ColorValue::Shade(Color::Black, ColorShade::S1))
        );
    }

    /// Raw CSS splits on `.` into a nonsense shade - parse must reject it on
    /// the color name, not trip `ColorShade::parse`'s debug assert.
    #[test]
    fn parse_rejects_raw_css_without_asserting_on_the_shade() {
        assert_eq!(ColorValue::parse("rgba(255, 255, 255, 0.15)"), None);
        assert_eq!(ColorValue::parse("1.5rem"), None);
    }

    #[test]
    fn parse_rejects_non_palette_values() {
        // Black/white have no shade scale, so only their bare names parse.
        assert_eq!(ColorValue::parse("black.3"), None);
        assert_eq!(ColorValue::parse("white-contrast"), None);
        assert_eq!(ColorValue::parse("primaryish"), None);
        assert_eq!(ColorValue::parse("#ff0000"), None);
    }
}
