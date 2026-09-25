/// A `RichTextEditor`'s toolbar: its name, its groups and one name per button.
///
/// ```
/// use libero::localization::RichTextEditorLabels;
///
/// const WORDS: RichTextEditorLabels = RichTextEditorLabels {
///     bold: "Fett",
///     ..RichTextEditorLabels::ENGLISH
/// };
/// assert_eq!(WORDS.bold, "Fett");
/// assert_eq!(WORDS.italic, "Italic");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RichTextEditorLabels {
    pub toolbar: &'static str,
    pub marks: &'static str,
    pub blocks: &'static str,
    pub history: &'static str,
    pub bold: &'static str,
    pub italic: &'static str,
    pub underline: &'static str,
    pub strike: &'static str,
    pub code: &'static str,
    pub bullet_list: &'static str,
    pub ordered_list: &'static str,
    pub quote: &'static str,
    pub code_block: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
}

impl RichTextEditorLabels {
    pub const ENGLISH: Self = Self {
        toolbar: "Formatting",
        marks: "Text style",
        blocks: "Blocks",
        history: "History",
        bold: "Bold",
        italic: "Italic",
        underline: "Underline",
        strike: "Strikethrough",
        code: "Inline code",
        bullet_list: "Bulleted list",
        ordered_list: "Numbered list",
        quote: "Quote",
        code_block: "Code block",
        undo: "Undo",
        redo: "Redo",
    };

    pub const GERMAN: Self = Self {
        toolbar: "Formatierung",
        marks: "Textstil",
        blocks: "Blöcke",
        history: "Verlauf",
        bold: "Fett",
        italic: "Kursiv",
        underline: "Unterstrichen",
        strike: "Durchgestrichen",
        code: "Code im Text",
        bullet_list: "Aufzählung",
        ordered_list: "Nummerierte Liste",
        quote: "Zitat",
        code_block: "Codeblock",
        undo: "Rückgängig",
        redo: "Wiederholen",
    };
}
