/// A `Menu` item's drawn shortcut hint. The spoken one is `aria-keyshortcuts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuLabels {
    /// Stands in for `Control`: "Control+S" draws "Ctrl+S".
    pub control: &'static str,
    /// Stands in for `Shift`: "Control+Shift+S" draws "Strg+Umschalt+S".
    pub shift: &'static str,
    pub alt: &'static str,
    pub meta: &'static str,
}

impl MenuLabels {
    pub const ENGLISH: Self = Self {
        control: "Ctrl",
        shift: "Shift",
        alt: "Alt",
        meta: "Meta",
    };
    pub const GERMAN: Self = Self {
        control: "Strg",
        shift: "Umschalt",
        alt: "Alt",
        meta: "Meta",
    };
}
