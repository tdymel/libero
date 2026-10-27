use dioxus::prelude::*;

use crate::{
    components::typography::Kbd,
    hooks::{Chord, ChordKey, chord_keys},
    localization::ShortcutHelpLabels,
};

/// [`chord_keys`] named in the localization's words, once.
pub(crate) fn chord_words(chord: &Chord, apple: bool, words: &ShortcutHelpLabels) -> Vec<String> {
    chord_keys(chord)
        .iter()
        .map(|key| {
            match key {
                ChordKey::Ctrl => words.ctrl,
                ChordKey::Alt if apple => words.option,
                ChordKey::Alt => words.alt,
                ChordKey::Shift => words.shift,
                ChordKey::Meta if apple => words.meta,
                ChordKey::Meta => words.meta_other,
                ChordKey::Space => words.space,
                ChordKey::Key(name) => words.key_name(name),
            }
            .to_string()
        })
        .collect()
}

/// A chord's keys from [`chord_words`], one [`Kbd`] each, joined by " + ".
pub(crate) fn chord_kbd(keys: Vec<String>) -> Element {
    let last = keys.len().saturating_sub(1);
    rsx! {
        for (index, key) in keys.into_iter().enumerate() {
            Kbd { "{key}" }
            if index < last { " + " }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(chord: &str, apple: bool, labels: &ShortcutHelpLabels) -> Vec<String> {
        chord_words(&Chord::parse(chord, apple).unwrap(), apple, labels)
    }

    /// Todo 2449: Space, Meta and the named keys follow the localization.
    #[test]
    fn a_chord_shows_the_localized_key_names() {
        let german = ShortcutHelpLabels::GERMAN;
        let keys = |chord| words(chord, false, &german);
        assert_eq!(keys("space"), ["Leertaste"]);
        assert_eq!(keys("meta+escape"), ["Meta", "Esc"]);
        assert_eq!(keys("ctrl+ArrowUp"), ["Strg", "Pfeil nach oben"]);
        // A key the table lacks keeps its `Key` name.
        assert_eq!(keys("f10"), ["F10"]);
    }

    #[test]
    fn english_names_the_keys_and_the_platforms_modifiers() {
        let english = ShortcutHelpLabels::ENGLISH;
        assert_eq!(words("ctrl+ArrowUp", false, &english), ["Ctrl", "Up Arrow"]);
        assert_eq!(words("escape", false, &english), ["Esc"]);
        assert_eq!(
            words("shift+PageDown", false, &english),
            ["Shift", "Page Down"]
        );
        assert_eq!(words("shift+mod+k", true, &english), ["Shift", "Cmd", "K"]);
        assert_eq!(words("alt+f10", true, &english), ["Option", "F10"]);
        assert_eq!(
            words("shift+mod+k", false, &english),
            ["Ctrl", "Shift", "K"]
        );
    }
}
