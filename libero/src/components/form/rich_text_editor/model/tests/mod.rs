//! Model tests. States are written as Markdown with `|` for the caret; two `|`
//! make a selection from the first to the second.

mod commands;
mod edit;
mod links;
mod markdown;
mod serde;

use super::doc::{Inline, split_inlines};
use super::{Doc, EditorState, Position, Selection};

pub(super) fn state(markdown: &str) -> EditorState {
    let mut doc = Doc::from_markdown(markdown);
    let mut found = Vec::new();
    for key in doc.leaves() {
        loop {
            let block = doc.get_mut(key).expect("a leaf");
            let Some(offset) = block.text().chars().position(|c| c == '|') else {
                break;
            };
            let (head, tail) = split_inlines(block.inlines(), offset);
            let (_, tail) = split_inlines(&tail, 1);
            *block.inlines_mut() = head.into_iter().chain(tail).collect();
            super::doc::normalize_inlines(block.inlines_mut());
            found.push(Position::new(key, offset));
        }
    }
    let selection = match found.as_slice() {
        [caret] => Selection::caret(*caret),
        [anchor, head] => Selection::range(*anchor, *head),
        _ => panic!("write one or two `|` in {markdown:?}"),
    };
    EditorState::new(doc).with_selection(selection)
}

/// The state as Markdown with `|` at the caret (or both selection ends).
pub(super) fn show(state: &EditorState) -> String {
    let mut doc = state.doc.clone();
    let (from, to) = state.ordered();
    let ends = if from == to {
        vec![from]
    } else {
        vec![to, from]
    };
    for at in ends {
        let block = doc.get_mut(at.block).expect("a leaf");
        if !block.is_leaf() || block.kind.content() != super::doc::ContentKind::Inline {
            continue;
        }
        let code = block.kind.is_code();
        let marks = match split_inlines(block.inlines(), at.offset) {
            _ if code => Default::default(),
            (head, _) if at.offset > 0 => match head.last() {
                Some(Inline::Text { marks, .. }) => marks.clone(),
                _ => Default::default(),
            },
            (_, tail) => match tail.first() {
                Some(Inline::Text { marks, .. }) => marks.clone(),
                _ => Default::default(),
            },
        };
        let (head, tail) = split_inlines(block.inlines(), at.offset);
        *block.inlines_mut() = head
            .into_iter()
            .chain([Inline::marked("|", marks)])
            .chain(tail)
            .collect();
        super::doc::normalize_inlines(block.inlines_mut());
    }
    doc.to_markdown().trim_end().to_string()
}

/// Runs `edit` on the state written as `before` and checks the result.
#[track_caller]
pub(super) fn check(before: &str, edit: impl FnOnce(&mut EditorState) -> bool, after: &str) {
    let mut state = state(before);
    edit(&mut state);
    assert_eq!(show(&state), after, "from {before:?}");
}

pub(super) fn md(doc: &Doc) -> String {
    doc.to_markdown().trim_end().to_string()
}
