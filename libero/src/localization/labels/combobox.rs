/// Every list with a search or typed filter: `Select`, `MultiSelect`, `Cascader`,
/// `Autocomplete`, `TagsField`. Its loader reads [`CommonLabels::loading`](super::CommonLabels::loading).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComboboxLabels {
    /// Shown, and announced, when typed text matches no option.
    pub nothing_found: &'static str,
}

impl ComboboxLabels {
    pub const ENGLISH: Self = Self {
        nothing_found: "No results",
    };

    pub const GERMAN: Self = Self {
        nothing_found: "Keine Ergebnisse",
    };
}
