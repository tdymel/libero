/// A `RichTextEditor`'s toolbar, link dialog, shortcut list and announcements.
/// `{level}` and `{name}` are filled in.
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
    /// The trigger of the menu holding the buttons a narrow toolbar has no room for.
    pub more: &'static str,
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
    pub link: &'static str,
    pub link_url: &'static str,
    pub link_save: &'static str,
    pub link_remove: &'static str,
    pub link_unsafe: &'static str,
    pub unlink: &'static str,
    pub shortcuts: &'static str,
    pub block_type: &'static str,
    pub paragraph: &'static str,
    /// `{level}` is 1 to 6.
    pub heading: &'static str,
    pub indent: &'static str,
    pub outdent: &'static str,
    pub hard_break: &'static str,
    pub rule: &'static str,
    /// Announced when a toggle turns on; `{name}` is the toggle's label.
    pub on: &'static str,
    /// Announced when a toggle turns off.
    pub off: &'static str,
}

impl RichTextEditorLabels {
    pub const ENGLISH: Self = Self {
        toolbar: "Formatting",
        marks: "Text style",
        blocks: "Blocks",
        history: "History",
        more: "More formatting",
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
        link: "Link",
        link_url: "URL",
        link_save: "Save",
        link_remove: "Remove link",
        link_unsafe: "Use an http, https or mailto link",
        unlink: "Remove link",
        shortcuts: "Keyboard shortcuts",
        block_type: "Text type",
        paragraph: "Paragraph",
        heading: "Heading {level}",
        indent: "Indent",
        outdent: "Outdent",
        hard_break: "Line break",
        rule: "Divider",
        on: "{name} on",
        off: "{name} off",
    };

    pub const GERMAN: Self = Self {
        toolbar: "Formatierung",
        marks: "Textstil",
        blocks: "Blöcke",
        history: "Verlauf",
        more: "Weitere Formatierung",
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
        link: "Link",
        link_url: "URL",
        link_save: "Speichern",
        link_remove: "Link entfernen",
        link_unsafe: "Nutze einen http-, https- oder mailto-Link",
        unlink: "Link entfernen",
        shortcuts: "Tastenkürzel",
        block_type: "Textart",
        paragraph: "Absatz",
        heading: "Überschrift {level}",
        indent: "Einrücken",
        outdent: "Ausrücken",
        hard_break: "Zeilenumbruch",
        rule: "Trennlinie",
        on: "{name} an",
        off: "{name} aus",
    };
}
