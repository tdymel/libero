/// A `ShortcutHelp`'s title and the names its chords show for the modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortcutHelpLabels {
    /// The dialog's title, unless its `title` prop is set.
    pub title: &'static str,
    pub ctrl: &'static str,
    pub alt: &'static str,
    pub shift: &'static str,
    /// Meta: Cmd on Apple platforms, where `mod` shows it too.
    pub meta: &'static str,
    /// Alt on Apple platforms.
    pub option: &'static str,
}

impl ShortcutHelpLabels {
    pub const ENGLISH: Self = Self {
        title: "Keyboard shortcuts",
        ctrl: "Ctrl",
        alt: "Alt",
        shift: "Shift",
        meta: "Cmd",
        option: "Option",
    };

    pub const GERMAN: Self = Self {
        title: "Tastenkürzel",
        ctrl: "Strg",
        alt: "Alt",
        shift: "Umschalt",
        meta: "Cmd",
        option: "Wahl",
    };
}
