/// An `Anchor`'s or link `Chip`'s cue for a link that opens a new tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorLabels {
    /// Read after the link text, hidden from sight: the icon shows it.
    pub new_tab: &'static str,
}

impl AnchorLabels {
    pub const ENGLISH: Self = Self {
        new_tab: "(opens in a new tab)",
    };

    pub const GERMAN: Self = Self {
        new_tab: "(öffnet in einem neuen Tab)",
    };
}
