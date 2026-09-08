use super::Theme;

/// The themes an app ships, and which of them is its light and its dark.
///
/// ```ignore
/// ThemeSet::new()
///     .with_name("Acme")
///     .light(&LIGHT)
///     .dark(&DARK)
///     .named("sepia", &SEPIA)
/// ```
///
/// The library ships several ready-made ones - [`ThemeSet::DEFAULT`] and the
/// community palettes beside it, all of them in
/// [`CATALOGUE`](Self::CATALOGUE).
///
/// The light/dark pair is emitted into the stylesheet up front, so switching
/// between the two is one attribute on the document root: no re-render to see
/// the new colours, right on the first paint, right under SSR, and
/// `prefers-color-scheme` works with no JS at all. A theme added with
/// [`named`](Self::named) is the rarer case and costs a rebuild of the sheet
/// when it is selected - which is what stops the sheet growing with every
/// theme an app happens to own.
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

    /// The library's own pair: [`Theme::DEFAULT`] as the light theme and
    /// [`Theme::DARK`] as the dark one. An app that authors no theme at all
    /// still gets a working dark scheme, and `prefers-color-scheme` alone
    /// switches it.
    pub const DEFAULT: ThemeSet = ThemeSet {
        name: "Libero",
        light: &Theme::DEFAULT,
        dark: Some(&Theme::DARK),
        extras: Vec::new(),
    };

    /// A named pair, which is what every set the library ships is. `const`,
    /// so a palette can be one.
    pub const fn pair(name: &'static str, light: &'static Theme, dark: &'static Theme) -> Self {
        Self {
            name,
            light,
            dark: Some(dark),
            extras: Vec::new(),
        }
    }

    pub fn new() -> Self {
        Self::DEFAULT
    }

    /// One theme and nothing else - what `LiberoProvider`'s `theme:` prop is
    /// sugar for. Deliberately not `new().light(theme)`: the default set
    /// carries [`Theme::DARK`], and an app that hands over one theme of its
    /// own has not authored a dark counterpart for it.
    pub fn of(theme: &'static Theme) -> Self {
        Self {
            name: "Custom",
            light: theme,
            dark: None,
            extras: Vec::new(),
        }
    }

    /// What a picker calls this set. Only a label - nothing resolves through
    /// it, and two sets may share one.
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

    /// A theme beyond the pair. `"light"` and `"dark"` are the pair's own
    /// names, so they set the pair rather than adding a third entry that
    /// could never be reached.
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

    /// Whether `name` is one of the two the sheet already carries. The pair
    /// switches by attribute; anything else rebuilds the sheet.
    pub(crate) fn is_in_pair(&self, name: &str) -> bool {
        name == Self::LIGHT || (name == Self::DARK && self.dark.is_some())
    }
}

impl Default for ThemeSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static OTHER: Theme = Theme::DEFAULT;

    #[test]
    fn a_bare_set_is_the_library_s_own_pair_and_nothing_else() {
        let set = ThemeSet::new();

        // By value, not by pointer: `Theme::DEFAULT` is an associated const,
        // so every `&Theme::DEFAULT` is its own promoted temporary and
        // `ptr::eq` on two of them is not required to hold.
        assert_eq!(set.light_theme(), &Theme::DEFAULT);
        assert_eq!(set.dark_theme(), Some(&Theme::DARK));
        assert!(set.get("sepia").is_none());
        assert!(set.is_in_pair("light"));
        assert!(set.is_in_pair("dark"));
    }

    /// One theme is one theme: an app that hands over a theme of its own has
    /// not authored a dark counterpart for it, so `of` must not quietly pair
    /// it with the library's.
    #[test]
    fn one_theme_carries_no_dark_half() {
        let set = ThemeSet::of(&OTHER);

        assert!(set.dark_theme().is_none());
        assert!(!set.is_in_pair("dark"));
    }

    #[test]
    fn the_pair_is_reachable_by_name() {
        let set = ThemeSet::new().dark(&OTHER);

        assert!(std::ptr::eq(set.get("dark").expect("a dark theme"), &OTHER));
        assert!(set.is_in_pair("dark"));
    }

    /// `"light"`/`"dark"` through `named` would otherwise add an entry that
    /// `get` could never reach, because the pair answers first.
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
