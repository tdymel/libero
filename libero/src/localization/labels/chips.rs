/// What a field holding chips announces when its list changes. `{labels}`,
/// `{added}` and `{removed}` are labels joined by `, `.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChipsLabels {
    pub added: &'static str,
    pub removed: &'static str,
    /// One change that both added and removed.
    pub added_and_removed: &'static str,
}

impl ChipsLabels {
    pub const ENGLISH: Self = Self {
        added: "Added {labels}",
        removed: "Removed {labels}",
        added_and_removed: "Added {added}. Removed {removed}",
    };

    pub const GERMAN: Self = Self {
        added: "{labels} hinzugefügt",
        removed: "{labels} entfernt",
        added_and_removed: "{added} hinzugefügt. {removed} entfernt",
    };
}
