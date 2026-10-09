//! What a `beforeinput` asks for, by its `inputType`. Everything the model does not
//! handle is cancelled; only composition reaches the DOM (it cannot be cancelled).

use super::model::{Builtin, Chord, EditorState, KeyPress};

/// The key that sends a [`RichTextEditor`](super::RichTextEditor)'s `onsubmit`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubmitOn {
    /// Enter sends and Shift+Enter breaks the line, as in a chat box.
    #[default]
    Enter,
    /// Mod+Enter (Cmd on Apple, Ctrl elsewhere) sends and Enter stays a new paragraph,
    /// as in a comment box.
    ModEnter,
}

impl SubmitOn {
    pub(crate) fn pressed(self, press: &KeyPress, apple: bool) -> bool {
        let chord = match self {
            Self::Enter => "Enter",
            Self::ModEnter => "Mod+Enter",
        };
        chord
            .parse::<Chord>()
            .is_ok_and(|chord| chord.matches(press, apple))
    }
}

/// What a [`RichTextEditor`](super::RichTextEditor)'s `intercept` sees before the editor acts.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditorInput {
    /// A key press, before the keymap. Android soft keyboards report most keys as
    /// `"Unidentified"`; Enter arrives as `Key("Enter")` all the same.
    Key(KeyPress),
    /// Typed text, before it is inserted. Composed text (IME, most Android typing) never
    /// arrives here: watch the doc through the handle for that.
    Text(String),
}

impl EditorInput {
    /// The key's name (`"ArrowDown"`, `"Enter"`, `"a"`), `None` for text.
    pub fn key(&self) -> Option<&str> {
        match self {
            Self::Key(press) => Some(&press.key),
            Self::Text(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Intent {
    Type(String),
    Run(Builtin),
    Delete(Delete),
    /// Leave it to the browser: composition, reconciled at `compositionend`.
    Pass,
    Cancel,
}

/// The word and line deletes, which have no [`Builtin`] of their own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Delete {
    WordBackward,
    WordForward,
    LineBackward,
    LineForward,
}

impl Delete {
    pub(crate) fn run(self, state: &mut EditorState) -> bool {
        match self {
            Self::WordBackward => state.delete_word_backward(),
            Self::WordForward => state.delete_word_forward(),
            Self::LineBackward => state.delete_line_backward(),
            Self::LineForward => state.delete_line_forward(),
        }
    }
}

pub(crate) fn intent(input_type: &str, data: Option<String>) -> Intent {
    use Builtin as B;
    match input_type {
        "insertText" => match data {
            Some(text) if !text.is_empty() => Intent::Type(text),
            _ => Intent::Cancel,
        },
        // A substitution's target is a range the event's `data` does not carry.
        "insertReplacementText" => Intent::Cancel,
        "insertCompositionText" => Intent::Pass,
        "insertParagraph" => Intent::Run(B::SplitBlock),
        "insertLineBreak" => Intent::Run(B::HardBreak),
        "deleteContentBackward" | "deleteByCut" | "deleteContent" => Intent::Run(B::DeleteBackward),
        "deleteContentForward" => Intent::Run(B::DeleteForward),
        "deleteWordBackward" => Intent::Delete(Delete::WordBackward),
        "deleteWordForward" => Intent::Delete(Delete::WordForward),
        // The model has no layout: a soft line ends at a hard break or the block's edge.
        "deleteSoftLineBackward" | "deleteHardLineBackward" => Intent::Delete(Delete::LineBackward),
        "deleteSoftLineForward" | "deleteHardLineForward" => Intent::Delete(Delete::LineForward),
        "formatBold" => Intent::Run(B::Bold),
        "formatItalic" => Intent::Run(B::Italic),
        "formatUnderline" => Intent::Run(B::Underline),
        "formatStrikeThrough" => Intent::Run(B::Strike),
        "historyUndo" => Intent::Run(B::Undo),
        "historyRedo" => Intent::Run(B::Redo),
        _ => Intent::Cancel,
    }
}

/// Enter pressed inside a composed word reaches the leaf as a trailing "\n": with
/// `enter_sends` it is split off from the text `dom` and `true` says Enter was pressed.
pub(crate) fn split_enter<'a>(old: &str, dom: &'a str, enter_sends: bool) -> (&'a str, bool) {
    match dom.strip_suffix('\n') {
        Some(rest) if enter_sends && !old.ends_with('\n') => (rest, true),
        _ => (dom, false),
    }
}

/// The char range of `old` that `new` replaced, and its replacement. `None` when equal.
pub(crate) fn text_diff(old: &str, new: &str) -> Option<(usize, usize, String)> {
    let (old, new): (Vec<char>, Vec<char>) = (old.chars().collect(), new.chars().collect());
    if old == new {
        return None;
    }
    let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let inserted = new[prefix..new.len() - suffix].iter().collect();
    Some((prefix, old.len() - suffix, inserted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_diff_is_the_changed_middle() {
        assert_eq!(text_diff("ab", "ab"), None);
        assert_eq!(text_diff("ac", "a日本c"), Some((1, 1, "日本".into())));
        assert_eq!(
            text_diff("helo world", "hello world"),
            Some((3, 3, "l".into()))
        );
        assert_eq!(text_diff("teh x", "the x"), Some((1, 3, "he".into())));
        assert_eq!(text_diff("aaa", "aa"), Some((2, 3, String::new())));
    }

    #[test]
    fn typing_inserts_its_data() {
        assert_eq!(
            intent("insertText", Some("a".into())),
            Intent::Type("a".into())
        );
        assert_eq!(intent("insertText", None), Intent::Cancel);
    }

    #[test]
    fn composition_passes_and_unknown_types_cancel() {
        assert_eq!(
            intent("insertCompositionText", Some("日".into())),
            Intent::Pass
        );
        assert_eq!(intent("insertFromDrop", None), Intent::Cancel);
        assert_eq!(intent("formatFontColor", None), Intent::Cancel);
    }

    #[test]
    fn structural_types_run_commands() {
        assert_eq!(
            intent("insertParagraph", None),
            Intent::Run(Builtin::SplitBlock)
        );
        assert_eq!(
            intent("deleteWordBackward", None),
            Intent::Delete(Delete::WordBackward)
        );
        assert_eq!(
            intent("deleteSoftLineBackward", None),
            Intent::Delete(Delete::LineBackward)
        );
        assert_eq!(intent("historyRedo", None), Intent::Run(Builtin::Redo));
    }

    #[test]
    fn a_composed_words_trailing_newline_is_enter_only_when_enter_sends() {
        assert_eq!(split_enter("hel", "hello\n", true), ("hello", true));
        assert_eq!(split_enter("hel", "hello\n", false), ("hello\n", false));
        assert_eq!(split_enter("hel", "hello", true), ("hello", false));
        assert_eq!(split_enter("hello\n", "hello\n", true), ("hello\n", false));
        assert_eq!(split_enter("", "\n", true), ("", true));
    }

    #[test]
    fn enter_and_mod_enter_send_by_their_mode() {
        let enter = KeyPress::new("Enter");
        let shifted = KeyPress {
            shift: true,
            ..enter.clone()
        };
        let ctrl = KeyPress {
            ctrl: true,
            ..enter.clone()
        };
        let cmd = KeyPress {
            meta: true,
            ..enter.clone()
        };
        assert!(SubmitOn::Enter.pressed(&enter, false));
        assert!(!SubmitOn::Enter.pressed(&shifted, false));
        assert!(!SubmitOn::Enter.pressed(&ctrl, false));
        assert!(!SubmitOn::ModEnter.pressed(&enter, false));
        assert!(SubmitOn::ModEnter.pressed(&ctrl, false));
        assert!(!SubmitOn::ModEnter.pressed(&cmd, false));
        assert!(SubmitOn::ModEnter.pressed(&cmd, true));
        assert!(!SubmitOn::ModEnter.pressed(&KeyPress::new("a"), false));
    }

    #[test]
    fn a_replacement_is_ignored_rather_than_typed_at_the_caret() {
        assert_eq!(
            intent("insertReplacementText", Some("the".into())),
            Intent::Cancel
        );
    }
}
