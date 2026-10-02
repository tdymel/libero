use super::combobox::{english_results, german_results};

/// Every string a `Spotlight` says to a reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
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
    /// Announced when a query leaves some actions. A fn rather than a template, for a
    /// language's plural forms.
    pub results: fn(usize) -> String,
}

impl SpotlightLabels {
    pub const ENGLISH: Self = Self {
        label: "Command palette",
        search: "Search commands",
        placeholder: "Search...",
        nothing_found: "Nothing found",
        loading: "Searching",
        results: english_results,
    };

    pub const GERMAN: Self = Self {
        label: "Befehlspalette",
        search: "Befehle durchsuchen",
        placeholder: "Suchen …",
        nothing_found: "Nichts gefunden",
        loading: "Suche läuft",
        results: german_results,
    };
}
