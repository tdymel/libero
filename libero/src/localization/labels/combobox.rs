/// `ComboboxLabels::ENGLISH.results`, also `SpotlightLabels::ENGLISH.results`.
pub(super) fn english_results(n: usize) -> String {
    match n {
        1 => "1 result".to_string(),
        n => format!("{n} results"),
    }
}

/// `ComboboxLabels::GERMAN.results`, also `SpotlightLabels::GERMAN.results`.
pub(super) fn german_results(n: usize) -> String {
    match n {
        1 => "1 Ergebnis".to_string(),
        n => format!("{n} Ergebnisse"),
    }
}

/// Every list with a search or typed filter: `Select`, `MultiSelect`, `Cascader`,
/// `Autocomplete`, `TagsField`. Its loader reads [`CommonLabels::loading`](super::CommonLabels::loading).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct ComboboxLabels {
    /// Shown, and announced, when typed text matches no option.
    pub nothing_found: &'static str,
    /// Announced when typed text leaves some options. A fn rather than a template,
    /// for a language's plural forms (WCAG 4.1.3).
    pub results: fn(usize) -> String,
}

impl ComboboxLabels {
    pub const ENGLISH: Self = Self {
        nothing_found: "No results",
        results: english_results,
    };

    pub const GERMAN: Self = Self {
        nothing_found: "Keine Ergebnisse",
        results: german_results,
    };
}
