/// Where the app's override is persisted on the web, and what the inline
/// restore script reads back before first paint. Part of the library's
/// contract, not an implementation detail: an app may write it itself.
pub const COLOR_SCHEME_STORAGE_KEY: &str = "lsx-color-scheme";

/// Restores a stored scheme onto the document root **before first paint**.
///
/// It exists because Rust cannot do this job: the wasm bundle has not run
/// when the first frame is painted, so a Rust-side write is one flash of the
/// wrong scheme too late. `LiberoProvider` does not emit it - a `<script>` a
/// renderer inserts into the DOM never executes, and it has to be in the
/// served `<head>` to beat the paint anyway. An app that wants no flash
/// pastes it into its own `index.html`:
///
/// ```html
/// <script>try{var s=localStorage.getItem('lsx-color-scheme');
/// if(s==='light'||s==='dark')document.documentElement.setAttribute('data-lsx-theme',s)}catch(e){}</script>
/// ```
///
/// Nothing else belongs in it. It writes one attribute, it swallows its own
/// errors - `localStorage` throws rather than returning nothing in a private
/// window with site data blocked - and it leaves the root untouched for the
/// system case, where the sheet's own media block is already right.
///
/// **It is inline, so a strict CSP needs its hash:**
/// `sha256-i2dSUtjYkvH3Km+1WnMbJX1ZfgYnbdyuKeoc3clqFQ0=`. The test below
/// pins the script, so the hash cannot silently stop matching.
pub const COLOR_SCHEME_RESTORE_SCRIPT: &str = "try{var s=localStorage.getItem('lsx-color-scheme');\
if(s==='light'||s==='dark')document.documentElement.setAttribute('data-lsx-theme',s)}catch(e){}";

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

    /// Nothing makes the script follow the three names it depends on - it is
    /// a literal, because `concat!` takes literals only. This does. It pins
    /// the script itself too, because a strict CSP allows it by hash: an edit
    /// here is an edit to a number published in the docs.
    #[test]
    fn the_restore_script_matches_the_names_it_depends_on() {
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(COLOR_SCHEME_STORAGE_KEY));
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(crate::theme::THEME_ATTRIBUTE));
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(ColorSchemeSetting::Light.as_str()));
        assert!(COLOR_SCHEME_RESTORE_SCRIPT.contains(ColorSchemeSetting::Dark.as_str()));
        // The system case leaves the root alone, so the sheet's own media
        // block answers - naming it here would be a bug, not an omission.
        assert!(!COLOR_SCHEME_RESTORE_SCRIPT.contains(ColorSchemeSetting::System.as_str()));

        assert_eq!(
            COLOR_SCHEME_RESTORE_SCRIPT,
            "try{var s=localStorage.getItem('lsx-color-scheme');\
if(s==='light'||s==='dark')document.documentElement.setAttribute('data-lsx-theme',s)}catch(e){}"
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
