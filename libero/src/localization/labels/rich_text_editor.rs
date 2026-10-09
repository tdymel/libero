use super::combobox::{english_results, german_results};

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
#[allow(
    unpredictable_function_pointer_comparisons,
    reason = "compares by address; a miss on a copied closure only re-renders"
)]
pub struct RichTextEditorLabels {
    pub toolbar: &'static str,
    pub marks: &'static str,
    pub blocks: &'static str,
    pub history: &'static str,
    /// The group of the caller's own toolbar buttons.
    pub custom_tools: &'static str,
    /// The trigger of the menu holding the buttons a narrow toolbar has no room for.
    pub more: &'static str,
    pub bold: &'static str,
    pub italic: &'static str,
    pub underline: &'static str,
    pub strike: &'static str,
    pub code: &'static str,
    pub bullet_list: &'static str,
    pub ordered_list: &'static str,
    pub task_list: &'static str,
    /// Checks or unchecks the task items at the caret; announced as on or off.
    pub toggle_task: &'static str,
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
    /// Leaves a code block for a new paragraph after it.
    pub exit_block: &'static str,
    /// The code block language picker, shown while the caret is in a code block.
    pub language: &'static str,
    /// The picker's choice for no language.
    pub plain_text: &'static str,
    /// The language button on a code block's opening fence, and its shortcut.
    pub code_language: &'static str,
    /// Announced when a toggle turns on; `{name}` is the toggle's label.
    pub on: &'static str,
    /// Announced when a toggle turns off.
    pub off: &'static str,
    /// Describes the text: how to leave it when Tab indents a list.
    pub leave_hint: &'static str,
    /// Announced when an `overlay` list's `overlay_results` changes. A fn, for plural forms.
    pub results: fn(usize) -> String,
    /// Announced when `overlay_results` drops to 0.
    pub nothing_found: &'static str,
}

impl RichTextEditorLabels {
    pub const ENGLISH: Self = Self {
        toolbar: "Formatting",
        marks: "Text style",
        blocks: "Blocks",
        history: "History",
        custom_tools: "More tools",
        more: "More formatting",
        bold: "Bold",
        italic: "Italic",
        underline: "Underline",
        strike: "Strikethrough",
        code: "Inline code",
        bullet_list: "Bulleted list",
        ordered_list: "Numbered list",
        task_list: "Task list",
        toggle_task: "Task done",
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
        exit_block: "Leave code block",
        language: "Language",
        plain_text: "Plain text",
        code_language: "Code language",
        on: "{name} on",
        off: "{name} off",
        leave_hint: "In a list, Tab indents. Press Escape, then Tab, to leave the editor.",
        results: english_results,
        nothing_found: "No results",
    };

    pub const GERMAN: Self = Self {
        toolbar: "Formatierung",
        marks: "Textstil",
        blocks: "Blöcke",
        history: "Verlauf",
        custom_tools: "Weitere Werkzeuge",
        more: "Weitere Formatierung",
        bold: "Fett",
        italic: "Kursiv",
        underline: "Unterstrichen",
        strike: "Durchgestrichen",
        code: "Code im Text",
        bullet_list: "Aufzählung",
        ordered_list: "Nummerierte Liste",
        task_list: "Aufgabenliste",
        toggle_task: "Aufgabe erledigt",
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
        exit_block: "Codeblock verlassen",
        language: "Sprache",
        plain_text: "Klartext",
        code_language: "Codesprache",
        on: "{name} an",
        off: "{name} aus",
        leave_hint: "In einer Liste rückt Tab ein. Escape, dann Tab verlässt den Editor.",
        results: german_results,
        nothing_found: "Keine Ergebnisse",
    };
}
