/// A `CodeBlock`'s copy button and header. The copy announcements are
/// [`CopyLabels`](super::CopyLabels)'.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeBlockLabels {
    /// The copy button's name.
    pub copy: &'static str,
    /// Read before a `diff` line that starts with `+`; the `+` is drawing only.
    pub added: &'static str,
    /// Read before a `diff` line that starts with `-`.
    pub removed: &'static str,
    /// The scroll region's name when the code overflows and has no known language.
    pub code: &'static str,
    /// The scroll region's name with a known language; `{language}` is its name.
    pub code_named: &'static str,
    /// The header's language name for `language: "text"`, no highlighting.
    pub plain_text: &'static str,
}

impl CodeBlockLabels {
    pub const ENGLISH: Self = Self {
        copy: "Copy code",
        added: "Added",
        removed: "Removed",
        code: "Code",
        code_named: "{language} code",
        plain_text: "Plain text",
    };

    pub const GERMAN: Self = Self {
        copy: "Code kopieren",
        added: "Hinzugefügt",
        removed: "Entfernt",
        code: "Code",
        code_named: "{language}-Code",
        plain_text: "Nur-Text",
    };
}
