/// Words many components share. A component with its own wording has a group
/// of its own instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommonLabels {
    /// Every close button: `Alert`, `Notifications`, `Lightbox`,
    /// `FloatingWindow`, `Dialog`.
    pub close: &'static str,
    /// A clearable field's x.
    pub clear: &'static str,
    /// An x that drops one item - a chip, a file. `{label}` names the item.
    pub remove: &'static str,
    /// What a loader announces while options are fetched.
    pub loading: &'static str,
    /// A search box's placeholder.
    pub search: &'static str,
}

impl CommonLabels {
    pub const ENGLISH: Self = Self {
        close: "Close",
        clear: "Clear",
        remove: "Remove {label}",
        loading: "Loading",
        search: "Search",
    };

    pub const GERMAN: Self = Self {
        close: "Schließen",
        clear: "Leeren",
        remove: "{label} entfernen",
        loading: "Wird geladen",
        search: "Suchen",
    };
}
