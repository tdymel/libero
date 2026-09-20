/// The `localStorage` key of the app's scheme override on the web, read back by
/// [`COLOR_SCHEME_RESTORE_SCRIPT`]. Public contract: an app may write it itself.
pub const COLOR_SCHEME_STORAGE_KEY: &str = "lsx-color-scheme";

/// Restores a stored scheme onto the document root **before first paint**, which the
/// wasm bundle runs too late for. Paste it into your `index.html`'s `<head>`:
///
/// ```html
/// <script>try{var s=localStorage.getItem('lsx-color-scheme');
/// if(s==='light'||s==='dark')document.documentElement.setAttribute('data-lsx-theme',s)}catch(e){}</script>
/// ```
///
/// `LiberoProvider` can't emit it: an inserted `<script>` never runs. A strict CSP needs its
/// hash: `sha256-i2dSUtjYkvH3Km+1WnMbJX1ZfgYnbdyuKeoc3clqFQ0=` (pinned by a test).
///
/// Docs: <https://libero-ui.dev/about/theming>
pub const COLOR_SCHEME_RESTORE_SCRIPT: &str = "try{var s=localStorage.getItem('lsx-color-scheme');\
if(s==='light'||s==='dark')document.documentElement.setAttribute('data-lsx-theme',s)}catch(e){}";

/// The resolved light or dark scheme on screen. What the app asked for, "system"
/// included, is a [`ColorSchemeSetting`]; [`use_color_scheme`](crate::hooks::use_color_scheme) resolves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    /// This scheme's theme name in a [`ThemeSet`](crate::theme::ThemeSet), and the
    /// document root's attribute value pinning it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// The other scheme.
    pub const fn flipped(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
        }
    }
}

/// The scheme the app asked for; [`System`](Self::System) still has to be resolved
/// against the platform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorSchemeSetting {
    /// Follow the platform: no root attribute, so the sheet's
    /// `prefers-color-scheme` block decides, right on first paint without JavaScript.
    #[default]
    System,
    Light,
    Dark,
}

impl ColorSchemeSetting {
    /// The pinned scheme, or `None` for [`System`](Self::System).
    pub const fn fixed(self) -> Option<ColorScheme> {
        match self {
            Self::System => None,
            Self::Light => Some(ColorScheme::Light),
            Self::Dark => Some(ColorScheme::Dark),
        }
    }

    /// The scheme on screen, given the platform's.
    pub const fn resolve(self, system: ColorScheme) -> ColorScheme {
        match self.fixed() {
            Some(scheme) => scheme,
            None => system,
        }
    }

    /// The stored spelling, as the restore script reads it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// An unrecognised string is [`System`](Self::System), never a wrong scheme.
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

    /// The script is a literal (`concat!` takes no consts), so this ties it to its names.
    /// It also pins the text: the CSP hash in the docs depends on it.
    #[test]
    fn the_restore_script_matches_the_names_it_depends_on() {
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(COLOR_SCHEME_STORAGE_KEY));
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(crate::theme::THEME_ATTRIBUTE));
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(ColorSchemeSetting::Light.as_str()));
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(ColorSchemeSetting::Dark.as_str()));
        // The system case must leave the root alone for the media block.
        assert!(!COLOR_SCHEME_RESTORE_SCRIPT.contains(ColorSchemeSetting::System.as_str()));

        assert_eq!(
            COLOR_SCHEME_RESTORE_SCRIPT,
            "try{var s=localStorage.getItem('lsx-color-scheme');\
if(s==='light'||s==='dark')document.documentElement.setAttribute('data-lsx-theme',s)}catch(e){}"
        );
    }

    /// A value from a later version or another writer must not paint a wrong scheme.
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
