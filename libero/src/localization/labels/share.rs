/// What a `ShareButton` is named and announces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShareLabels {
    /// The icon-only button's name, unless its `aria_label` replaces it.
    pub share: &'static str,
    /// Announced once the link went to the clipboard, where no share sheet exists.
    pub copied: &'static str,
    pub share_failed: &'static str,
}

impl ShareLabels {
    pub const ENGLISH: Self = Self {
        share: "Share",
        copied: "Copied",
        share_failed: "Share failed",
    };

    pub const GERMAN: Self = Self {
        share: "Teilen",
        copied: "Kopiert",
        share_failed: "Teilen fehlgeschlagen",
    };
}
