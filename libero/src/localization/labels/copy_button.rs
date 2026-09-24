/// What a `CopyButton` is named and announces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyButtonLabels {
    /// The button's name, unless its `aria_label` replaces it.
    pub copy: &'static str,
    /// Announced once the copy landed.
    pub copied: &'static str,
    pub copy_failed: &'static str,
}

impl CopyButtonLabels {
    pub const ENGLISH: Self = Self {
        copy: "Copy",
        copied: "Copied",
        copy_failed: "Copy failed",
    };

    pub const GERMAN: Self = Self {
        copy: "Kopieren",
        copied: "Kopiert",
        copy_failed: "Kopieren fehlgeschlagen",
    };
}
