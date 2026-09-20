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
