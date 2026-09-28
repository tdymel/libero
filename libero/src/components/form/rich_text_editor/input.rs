//! What a `beforeinput` asks for, by its `inputType`. Everything the model does not
//! handle is cancelled; only composition reaches the DOM (it cannot be cancelled).

use super::model::{Builtin, KeyPress};

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
    /// Leave it to the browser: composition, reconciled at `compositionend`.
    Pass,
    Cancel,
}

pub(crate) fn intent(input_type: &str, data: Option<String>) -> Intent {
    use Builtin as B;
    match input_type {
        "insertText" | "insertReplacementText" => match data {
            Some(text) if !text.is_empty() => Intent::Type(text),
            _ => Intent::Cancel,
        },
        "insertCompositionText" => Intent::Pass,
        "insertParagraph" => Intent::Run(B::SplitBlock),
        "insertLineBreak" => Intent::Run(B::HardBreak),
        "deleteContentBackward"
        | "deleteWordBackward"
        | "deleteSoftLineBackward"
        | "deleteHardLineBackward"
        | "deleteByCut"
        | "deleteContent" => Intent::Run(B::DeleteBackward),
        "deleteContentForward"
        | "deleteWordForward"
        | "deleteSoftLineForward"
        | "deleteHardLineForward" => Intent::Run(B::DeleteForward),
        "formatBold" => Intent::Run(B::Bold),
        "formatItalic" => Intent::Run(B::Italic),
        "formatUnderline" => Intent::Run(B::Underline),
        "formatStrikeThrough" => Intent::Run(B::Strike),
        "historyUndo" => Intent::Run(B::Undo),
        "historyRedo" => Intent::Run(B::Redo),
        _ => Intent::Cancel,
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
            Intent::Run(Builtin::DeleteBackward)
        );
        assert_eq!(intent("historyRedo", None), Intent::Run(Builtin::Redo));
    }
}
