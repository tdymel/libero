/// `TextareaLabels::ENGLISH.characters_left`. A named fn, so every copy holds
/// the same address and compares equal.
fn english_characters_left(n: usize) -> String {
    match n {
        1 => "1 character left".to_string(),
        n => format!("{n} characters left"),
    }
}

/// `TextareaLabels::GERMAN.characters_left`. "Zeichen" is its own plural.
fn german_characters_left(n: usize) -> String {
    format!("Noch {n} Zeichen")
}
/// A `Textarea`'s `counter`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct TextareaLabels {
    /// Announced near `maxlength`: how many characters are left. A fn rather
    /// than a template, for a language's plural forms.
    ///
    /// ```
    /// use libero::localization::TextareaLabels;
    ///
    /// const WORDS: TextareaLabels = TextareaLabels {
    ///     characters_left: |n| match n {
    ///         1 => "Nur noch 1 Zeichen".to_string(),
    ///         n => format!("Nur noch {n} Zeichen"),
    ///     },
    /// };
    /// assert_eq!((WORDS.characters_left)(3), "Nur noch 3 Zeichen");
    /// assert_eq!((TextareaLabels::GERMAN.characters_left)(1), "Noch 1 Zeichen");
    /// assert_eq!((TextareaLabels::ENGLISH.characters_left)(1), "1 character left");
    /// ```
    pub characters_left: fn(usize) -> String,
}

impl TextareaLabels {
    pub const ENGLISH: Self = Self {
        characters_left: english_characters_left,
    };

    pub const GERMAN: Self = Self {
        characters_left: german_characters_left,
    };
}
