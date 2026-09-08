/// Where the app's override is persisted on the web, and what the inline
/// restore script reads back before first paint. Part of the library's
/// contract, not an implementation detail: an app may write it itself.
pub const COLOR_SCHEME_STORAGE_KEY: &str = "lsx-color-scheme";

/// Which end of the greyscale a page is drawn at.
///
/// The resolved answer, never "whatever the system says": that is a
/// [`ColorSchemeSetting`], and resolving it is
/// [`use_color_scheme`](crate::hooks::use_color_scheme)'s job.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    /// The name the theme of this scheme answers to in a
    /// [`ThemeSet`](crate::theme::ThemeSet), and the value the document root
    /// carries to pin it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub const fn flipped(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

/// What the app asked for, which is not the same question as which scheme is
/// on screen: [`System`](Self::System) has to be resolved against the
/// platform before anything can be painted in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorSchemeSetting {
    /// Follow the platform. The document root carries no theme attribute, so
    /// the sheet's own `@media (prefers-color-scheme: dark)` block decides -
    /// which is why this case needs no JavaScript to be right on first paint.
    #[default]
    System,
    Light,
    Dark,
}

impl ColorSchemeSetting {
    /// The scheme this setting means on a platform whose own preference is
    /// `system`. `None` only for [`System`](Self::System) itself, which is
    /// the one case that cannot answer for itself.
    pub const fn fixed(self) -> Option<ColorScheme> {
        match self {
            Self::System => None,
            Self::Light => Some(ColorScheme::Light),
            Self::Dark => Some(ColorScheme::Dark),
        }
    }

    pub const fn resolve(self, system: ColorScheme) -> ColorScheme {
        match self.fixed() {
            Some(scheme) => scheme,
            None => system,
        }
    }

    /// How the setting is spelled where it is persisted, and in the inline
    /// script that reads it back before first paint.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// An unrecognised string is [`System`](Self::System): a stored value
    /// this build does not know is not a reason to paint the wrong scheme.
    pub fn parse(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
}

impl From<ColorScheme> for ColorSchemeSetting {
    fn from(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Self::Light,
            ColorScheme::Dark => Self::Dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_setting_resolves_against_the_system_only_when_it_has_to() {
        assert_eq!(
            ColorSchemeSetting::Light.resolve(ColorScheme::Dark),
            ColorScheme::Light
        );
        assert_eq!(
            ColorSchemeSetting::System.resolve(ColorScheme::Dark),
            ColorScheme::Dark
        );
    }

    /// A value written by a later version, or by something else entirely,
    /// must not paint a scheme nobody asked for.
    #[test]
    fn an_unknown_stored_value_is_the_system_setting() {
        assert_eq!(
            ColorSchemeSetting::parse("sepia"),
            ColorSchemeSetting::System
        );
        assert_eq!(ColorSchemeSetting::parse(""), ColorSchemeSetting::System);
        assert_eq!(ColorSchemeSetting::parse("dark"), ColorSchemeSetting::Dark);
    }
}
