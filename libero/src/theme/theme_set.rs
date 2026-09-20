use super::Theme;

/// The themes an app ships, and which of them is its light and its dark.
/// Ready-made sets are in [`CATALOGUE`](Self::CATALOGUE).
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::LiberoProvider;
/// # use libero::theme::{HexColor, Theme, ThemeSet};
/// static LIGHT: Theme = Theme { primary: HexColor::new(0x0B7285), ..Theme::DEFAULT };
/// static DARK: Theme = Theme { primary: HexColor::new(0x0B7285), ..Theme::DARK };
///
/// # fn app() -> Element {
/// rsx! {
///     LiberoProvider { themes: ThemeSet::new().with_name("Acme").light(&LIGHT).dark(&DARK),
///         "..."
///     }
/// }
/// # }
/// ```
///
/// The light/dark pair ships in the sheet up front, so switching them is one root
/// attribute, right on first paint; a [`named`](Self::named) theme rebuilds the sheet.
///
/// Docs: <https://libero-ui.dev/about/theming>
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeSet {
    name: &'static str,
    light: &'static Theme,
    dark: Option<&'static Theme>,
    extras: Vec<(&'static str, &'static Theme)>,
}

impl ThemeSet {
    /// The name the light half of the pair answers to.
    pub const LIGHT: &'static str = "light";
    /// The name the dark half of the pair answers to.
    pub const DARK: &'static str = "dark";

    /// The library's own pair, [`Theme::DEFAULT`] and [`Theme::DARK`], switched by
    /// `prefers-color-scheme` alone.
    pub const DEFAULT: ThemeSet = ThemeSet {
        name: "Libero",
        light: &Theme::DEFAULT,
        dark: Some(&Theme::DARK),
        extras: Vec::new(),
    };

    /// A named light/dark pair, `const` so a palette can be one.
    pub const fn pair(name: &'static str, light: &'static Theme, dark: &'static Theme) -> Self {
        Self {
            name,
            light,
            dark: Some(dark),
            extras: Vec::new(),
        }
    }

    /// [`ThemeSet::DEFAULT`], to build on.
    pub fn new() -> Self {
        Self::DEFAULT
    }

    /// One theme, no dark half: what `LiberoProvider { themes: &MINE }` means. Not
    /// `new().light(theme)`, which would pair it with the library's [`Theme::DARK`].
    pub fn of(theme: &'static Theme) -> Self {
        Self {
            name: "Custom",
            light: theme,
            dark: None,
            extras: Vec::new(),
        }
    }

    /// What a picker calls this set; only a label.
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn light(mut self, theme: &'static Theme) -> Self {
        self.light = theme;
        self
    }

    pub fn dark(mut self, theme: &'static Theme) -> Self {
        self.dark = Some(theme);
        self
    }

    /// A theme beyond the pair. `"light"` and `"dark"` set the pair instead.
    pub fn named(mut self, name: &'static str, theme: &'static Theme) -> Self {
        match name {
            Self::LIGHT => return self.light(theme),
            Self::DARK => return self.dark(theme),
            _ => {}
        }

        match self.extras.iter_mut().find(|(known, _)| *known == name) {
            Some(entry) => entry.1 = theme,
            None => self.extras.push((name, theme)),
        }
        self
    }

    pub fn light_theme(&self) -> &'static Theme {
        self.light
    }

    pub fn dark_theme(&self) -> Option<&'static Theme> {
        self.dark
    }

    /// The theme `name` selects, or `None` if the set has no such theme.
    pub fn get(&self, name: &str) -> Option<&'static Theme> {
        match name {
            Self::LIGHT => Some(self.light),
            Self::DARK => self.dark,
            _ => self
                .extras
                .iter()
                .find(|(known, _)| *known == name)
                .map(|(_, theme)| *theme),
        }
    }

    /// Whether `name` is in the pair, which switches by attribute rather than a rebuild.
    pub(crate) fn is_in_pair(&self, name: &str) -> bool {
        name == Self::LIGHT || (name == Self::DARK && self.dark.is_some())
    }
}

impl Default for ThemeSet {
    fn default() -> Self {
        Self::new()
    }
}

/// [`ThemeSet::of`], except that [`Theme::DEFAULT`] brings its [`Theme::DARK`] along.
impl From<&'static Theme> for ThemeSet {
    fn from(theme: &'static Theme) -> Self {
        if theme == &Theme::DEFAULT {
            Self::DEFAULT
        } else {
            Self::of(theme)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static OTHER: Theme = Theme::DEFAULT;

    #[test]
    fn a_bare_set_is_the_library_s_own_pair_and_nothing_else() {
        let set = ThemeSet::new();

        // By value: each `&Theme::DEFAULT` is its own promoted temporary.
        assert_eq!(set.light_theme(), &Theme::DEFAULT);
        assert_eq!(set.dark_theme(), Some(&Theme::DARK));
        assert!(set.get("sepia").is_none());
        assert!(set.is_in_pair("light"));
        assert!(set.is_in_pair("dark"));
    }

    /// `of` must not quietly pair an app's theme with the library's dark one.
    #[test]
    fn one_theme_carries_no_dark_half() {
        let set = ThemeSet::of(&OTHER);

        assert!(set.dark_theme().is_none());
        assert!(!set.is_in_pair("dark"));
    }

    /// What `LiberoProvider { themes: &MINE }` gets: the library's own theme
    /// keeps its dark half, any other stands alone.
    #[test]
    fn a_lone_theme_converts_to_a_set_of_it_unless_it_is_the_default() {
        static MINE: Theme = Theme {
            primary: super::super::HexColor::new(0x7C3AED),
            ..Theme::DEFAULT
        };

        assert_eq!(ThemeSet::from(&Theme::DEFAULT), ThemeSet::DEFAULT);
        let set = ThemeSet::from(&MINE);
        assert!(std::ptr::eq(set.light_theme(), &MINE));
        assert!(set.dark_theme().is_none());
    }

    #[test]
    fn the_pair_is_reachable_by_name() {
        let set = ThemeSet::new().dark(&OTHER);

        assert!(std::ptr::eq(set.get("dark").expect("a dark theme"), &OTHER));
        assert!(set.is_in_pair("dark"));
    }

    /// Otherwise the entry would be unreachable: `get` asks the pair first.
    #[test]
    fn naming_a_theme_light_or_dark_sets_the_pair() {
        let set = ThemeSet::new().named("dark", &OTHER).named("sepia", &OTHER);

        assert!(set.dark_theme().is_some());
        assert!(set.get("sepia").is_some());
        assert!(!set.is_in_pair("sepia"));
    }

    #[test]
    fn naming_the_same_extra_twice_replaces_it() {
        let set = ThemeSet::new()
            .named("sepia", &Theme::DEFAULT)
            .named("sepia", &OTHER);

        assert!(set.get("sepia").is_some());
        assert_eq!(set.extras.len(), 1);
    }
}
