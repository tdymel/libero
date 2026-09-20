/// Which way a reader asked the contrast to go, as `prefers-contrast` says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Contrast {
    #[default]
    NoPreference,
    More,
    Less,
}

impl Contrast {
    /// The `prefers-contrast` value.
    pub fn as_str(self) -> &'static str {
        match self {
            Contrast::NoPreference => "no-preference",
            Contrast::More => "more",
            Contrast::Less => "less",
        }
    }
}

/// What the accessibility media features answer: `prefers-reduced-motion`,
/// `forced-colors`, `prefers-contrast` and `prefers-reduced-transparency`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AccessibilityPreferences {
    pub reduced_motion: bool,
    pub forced_colors: bool,
    pub contrast: Contrast,
    pub reduced_transparency: bool,
}

/// An app's own answer to the accessibility media features, over the system's.
/// `None` follows the system. Native renderers only: on the web the browser's
/// media queries decide.
///
/// ```ignore
/// LiberoProvider {
///     accessibility: AccessibilityOverrides {
///         reduced_motion: Some(true),
///         ..Default::default()
///     },
///     App {}
/// }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AccessibilityOverrides {
    pub reduced_motion: Option<bool>,
    pub forced_colors: Option<bool>,
    pub contrast: Option<Contrast>,
    pub reduced_transparency: Option<bool>,
}

impl AccessibilityOverrides {
    /// `system`, with every set override in its place.
    pub fn resolve(self, system: AccessibilityPreferences) -> AccessibilityPreferences {
        AccessibilityPreferences {
            reduced_motion: self.reduced_motion.unwrap_or(system.reduced_motion),
            forced_colors: self.forced_colors.unwrap_or(system.forced_colors),
            contrast: self.contrast.unwrap_or(system.contrast),
            reduced_transparency: self
                .reduced_transparency
                .unwrap_or(system.reduced_transparency),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_override_wins_and_the_rest_follow_the_system() {
        let system = AccessibilityPreferences {
            reduced_motion: true,
            contrast: Contrast::More,
            ..Default::default()
        };
        let overrides = AccessibilityOverrides {
            reduced_motion: Some(false),
            forced_colors: Some(true),
            ..Default::default()
        };
        assert_eq!(
            overrides.resolve(system),
            AccessibilityPreferences {
                reduced_motion: false,
                forced_colors: true,
                contrast: Contrast::More,
                reduced_transparency: false,
            }
        );
    }
}
