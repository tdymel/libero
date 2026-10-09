//! Paste: Markdown read back into blocks at the caret (todo 1258).

use super::doc::{Block, BlockKind, ContentKind, Doc, Inline, NodeKey};
use super::mark::Mark;
use super::state::{EditorState, Position};
use super::syntax::autolink;

impl EditorState {
    /// Pastes `text` at the caret, replacing the selection. `markdown` is the editor's
    /// own copy; other text is read a paragraph per line. A code block takes it raw.
    /// A leading and a trailing paragraph join the text around the caret, as typed.
    pub(crate) fn paste(&mut self, text: &str, markdown: bool) -> bool {
        let text = text.replace("\r\n", "\n");
        if let Some(changed) = self.paste_url(text.trim()) {
            return changed;
        }
        let mut changed = self.delete_selection();
        if self.block_kind().is_code() {
            return self.insert_text(&text) || changed;
        }
        let fragment = match markdown {
            true => Doc::from_markdown(&text),
            false => Doc::from_pasted(&text),
        };
        let mut blocks = fragment.blocks;
        let paragraph =
            |block: Option<&Block>| block.is_some_and(|b| b.kind == BlockKind::Paragraph);
        let inline = self.block(self.caret().block).kind.content() == ContentKind::Inline;

        if inline && paragraph(blocks.first()) {
            for piece in blocks.remove(0).inlines() {
                changed |= self.insert_inline(piece.clone());
            }
        }
        if blocks.is_empty() {
            return changed;
        }
        let tail = (inline && paragraph(blocks.last()))
            .then(|| blocks.pop())
            .flatten();
        for block in &mut blocks {
            self.rekey(block);
        }

        // The rest goes before the text after the caret, split off into its own block.
        let at = self.caret();
        let len = self.block(at.block).len();
        let (anchor, before) = match inline {
            true if at.offset > 0 && at.offset <= len && len > 0 => {
                self.split_block();
                (self.caret().block, true)
            }
            true => (at.block, true),
            false => (at.block, at.offset == 0),
        };
        let last = blocks.last().map(|block| block.key);
        match before {
            true => blocks
                .into_iter()
                .for_each(|block| self.insert_after(anchor, true, block)),
            false => blocks
                .into_iter()
                .rev()
                .for_each(|block| self.insert_after(anchor, false, block)),
        }
        match (tail, last) {
            (Some(tail), _) => {
                self.set_caret(Position::new(anchor, 0));
                for piece in tail.inlines() {
                    self.insert_inline(piece.clone());
                }
            }
            (None, Some(last)) => {
                if inline && self.block(anchor).is_empty() {
                    self.remove(anchor);
                }
                self.prune();
                self.caret_after(last);
            }
            (None, None) => {}
        }
        true
    }

    /// A pasted bare URL links the selection, or becomes a link at the caret. `None`
    /// when `text` is not just a URL or the paste goes to a code block.
    fn paste_url(&mut self, text: &str) -> Option<bool> {
        let chars: Vec<char> = text.chars().collect();
        let (_, href) = autolink(&chars).filter(|(len, _)| *len == chars.len())?;
        let kind = self.block_kind();
        if kind.is_code() || kind.content() != ContentKind::Inline {
            return None;
        }
        if !self.selection.is_collapsed() {
            return Some(self.set_link(href.as_str()).unwrap_or(false));
        }
        let mark = Mark::Link { href, title: None };
        let marks = self.current_marks().with(mark);
        Some(self.insert_inline(Inline::marked(text, marks)))
    }

    fn rekey(&mut self, block: &mut Block) {
        block.key = self.doc.key();
        if !block.is_leaf() {
            for child in block.children_mut() {
                self.rekey(child);
            }
        }
    }

    /// The caret at the end of `key`'s last text, else at the next line to type in.
    fn caret_after(&mut self, key: NodeKey) {
        let mut block = self.block(key);
        while let Some(child) = block.children().last() {
            block = child;
        }
        let leaf = block.key;
        if block.kind.content() == ContentKind::Inline {
            let end = block.len();
            self.set_caret(Position::new(leaf, end));
            return;
        }
        let leaves = self.doc.leaves();
        let next = leaves.get(self.leaf_index(leaf) + 1).copied();
        match next.filter(|next| self.block(*next).kind.content() == ContentKind::Inline) {
            Some(next) => self.set_caret(Position::new(next, 0)),
            None => {
                let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
                let paragraph_key = paragraph.key;
                self.insert_after(leaf, false, paragraph);
                self.set_caret(Position::new(paragraph_key, 0));
            }
        }
    }
}
