use dioxus::prelude::*;

use crate::{components::typography::Kbd, hooks::chord_keys, localization::ShortcutHelpLabels};

/// [`chord_keys`] with `Space`, `Meta` and the named keys in the localization's words.
pub(crate) fn chord_words(
    chord: &str,
    apple: bool,
    words: &ShortcutHelpLabels,
) -> Option<Vec<String>> {
    let keys = chord_keys(chord, apple, words)?;
    Some(keys.iter().map(|key| words.key_word(key)).collect())
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

    /// Todo 2449: Space, Meta and the named keys follow the localization.
    #[test]
    fn a_chord_shows_the_localized_key_names() {
        let words = ShortcutHelpLabels::GERMAN;
        let keys = |chord| chord_words(chord, false, &words).unwrap();
        assert_eq!(keys("space"), ["Leertaste"]);
        assert_eq!(keys("meta+escape"), ["Meta", "Esc"]);
        assert_eq!(keys("ctrl+ArrowUp"), ["Strg", "Pfeil nach oben"]);
        // A key the table lacks keeps its `Key` name.
        assert_eq!(keys("f10"), ["F10"]);
        let english = chord_words("ctrl+ArrowUp", false, &ShortcutHelpLabels::ENGLISH);
        assert_eq!(english.unwrap(), ["Ctrl", "ArrowUp"]);
    }
}
