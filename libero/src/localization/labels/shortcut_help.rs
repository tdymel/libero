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
    /// Meta off Apple platforms.
    pub meta_other: &'static str,
    pub space: &'static str,
    /// Named keys by their `Key` name (`ArrowUp`, `Escape`); a key missing here
    /// shows that name. The slice replaces the whole table, so list every key to name.
    ///
    /// ```
    /// use libero::localization::ShortcutHelpLabels;
    ///
    /// static LABELS: ShortcutHelpLabels = ShortcutHelpLabels {
    ///     key_names: &[("Escape", "Échap"), ("Enter", "Entrée")],
    ///     ..ShortcutHelpLabels::ENGLISH
    /// };
    /// ```
    pub key_names: &'static [(&'static str, &'static str)],
}

impl ShortcutHelpLabels {
    pub const ENGLISH: Self = Self {
        title: "Keyboard shortcuts",
        ctrl: "Ctrl",
        alt: "Alt",
        shift: "Shift",
        meta: "Cmd",
        option: "Option",
        meta_other: "Meta",
        space: "Space",
        key_names: &[
            ("Escape", "Esc"),
            ("Delete", "Del"),
            ("Insert", "Ins"),
            ("PageUp", "Page Up"),
            ("PageDown", "Page Down"),
            ("ArrowUp", "Up Arrow"),
            ("ArrowDown", "Down Arrow"),
            ("ArrowLeft", "Left Arrow"),
            ("ArrowRight", "Right Arrow"),
        ],
    };

    pub const GERMAN: Self = Self {
        title: "Tastenkürzel",
        ctrl: "Strg",
        alt: "Alt",
        shift: "Umschalt",
        meta: "Cmd",
        option: "Wahl",
        meta_other: "Meta",
        space: "Leertaste",
        key_names: &[
            ("Escape", "Esc"),
            ("Enter", "Eingabe"),
            ("Backspace", "Rücktaste"),
            ("Delete", "Entf"),
            ("Insert", "Einfg"),
            ("Home", "Pos1"),
            ("End", "Ende"),
            ("PageUp", "Bild auf"),
            ("PageDown", "Bild ab"),
            ("ArrowUp", "Pfeil nach oben"),
            ("ArrowDown", "Pfeil nach unten"),
            ("ArrowLeft", "Pfeil nach links"),
            ("ArrowRight", "Pfeil nach rechts"),
        ],
    };

    /// What a named key shows, from its `Key` name; a name missing here shows as it is.
    pub(crate) fn key_name<'a>(&self, key: &'a str) -> &'a str {
        self.key_names
            .iter()
            .find(|(name, _)| *name == key)
            .map_or(key, |(_, word)| *word)
    }
}
