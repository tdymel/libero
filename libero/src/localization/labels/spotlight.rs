/// Every string a `Spotlight` says to a reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpotlightLabels {
    /// Names the dialog, unless `SpotlightOptions::aria_label` does.
    pub label: &'static str,
    /// Names the search box.
    pub search: &'static str,
    pub placeholder: &'static str,
    /// Shown, and announced, when a query matches nothing.
    pub nothing_found: &'static str,
    /// Announced while `SpotlightOptions::loading` is set.
    pub loading: &'static str,
}

impl SpotlightLabels {
    pub const ENGLISH: Self = Self {
        label: "Command palette",
        search: "Search commands",
        placeholder: "Search...",
        nothing_found: "Nothing found",
        loading: "Searching",
    };

    pub const GERMAN: Self = Self {
        label: "Befehlspalette",
        search: "Befehle durchsuchen",
        placeholder: "Suchen …",
        nothing_found: "Nichts gefunden",
        loading: "Suche läuft",
    };
}
