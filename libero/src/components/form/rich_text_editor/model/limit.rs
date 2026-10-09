//! A length limit over the doc's plain text, counted as [`Doc::plain_text`] chars (todo 2715).

use super::doc::{ContentKind, NodeKey};
use super::state::{EditorState, Position, Selection};

impl EditorState {
    /// How many chars still fit under `max` once the selection is replaced.
    pub(crate) fn room(&self, max: usize) -> usize {
        if self.selection.is_collapsed() {
            return max.saturating_sub(self.doc.plain_len());
        }
        let mut after = self.clone();
        after.delete_selection();
        max.saturating_sub(after.doc.plain_len())
    }

    /// Cuts what sticks out past `max` before the caret, at most `at_most` chars.
    pub(crate) fn cut_over(&mut self, max: usize, at_most: usize) -> bool {
        let over = self.doc.plain_len().saturating_sub(max).min(at_most);
        over > 0 && self.cut_before_caret(over)
    }

    /// [`paste`](Self::paste) cut to `max` chars of plain text, if there is a limit.
    pub(crate) fn paste_within(&mut self, text: &str, markdown: bool, max: Option<usize>) -> bool {
        let changed = self.paste(text, markdown);
        if let Some(max) = max {
            self.cut_over(max, usize::MAX);
        }
        changed
    }

    /// Deletes the `chars` chars of plain text before the caret; the line break between
    /// two blocks counts one. `false` when there is nothing to cut.
    pub(crate) fn cut_before_caret(&mut self, chars: usize) -> bool {
        let caret = self.caret();
        let leaves: Vec<NodeKey> = self
            .doc
            .leaves()
            .into_iter()
            .filter(|key| self.block(*key).kind.content() == ContentKind::Inline)
            .collect();
        let Some(mut index) = leaves.iter().position(|key| *key == caret.block) else {
            return false;
        };
        let (mut left, mut offset) = (chars, caret.offset);
        let from = loop {
            if left <= offset {
                break Position::new(leaves[index], offset - left);
            }
            if index == 0 {
                break Position::new(leaves[0], 0);
            }
            left -= offset + 1;
            index -= 1;
            offset = self.block(leaves[index]).len();
        };
        if from == caret {
            return false;
        }
        self.select(Selection::range(from, caret));
        self.delete_selection()
    }
}
