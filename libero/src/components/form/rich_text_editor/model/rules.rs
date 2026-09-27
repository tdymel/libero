//! Markdown shortcuts: typed syntax turns into structure (`# ` a heading, `**x**` bold).
//! The recognizers live in [`super::syntax`], shared with the Markdown reader.

use super::doc::{BlockKind, ContentKind, Inline, slice_inlines};
use super::state::{EditorState, Position};
use super::syntax::{BlockSyntax, DELIMITERS, block_syntax, closing_span};

impl EditorState {
    /// Applies the shortcut the text just typed completes. Never in code blocks.
    pub fn apply_input_rules(&mut self) -> bool {
        if !self.selection.is_collapsed() {
            return false;
        }
        let block = self.block(self.caret().block);
        if block.kind.content() != ContentKind::Inline || block.kind.is_code() {
            return false;
        }
        self.block_rule() || self.inline_rule()
    }

    /// A prefix typed at the start of a paragraph, ended by a space.
    fn block_rule(&mut self) -> bool {
        let at = self.caret();
        let block = self.block(at.block);
        if block.kind != BlockKind::Paragraph {
            return false;
        }
        let before = slice_inlines(block.inlines(), 0, at.offset);
        let [Inline::Text { text, .. }] = before.as_slice() else {
            return false;
        };
        let Some(syntax) = text.strip_suffix(' ').and_then(block_syntax) else {
            return false;
        };
        self.delete_range(Position::new(at.block, 0), at);
        match syntax {
            BlockSyntax::Heading(level) => self.set_kind(at.block, BlockKind::heading(level)),
            BlockSyntax::Bullet(_) => self.toggle_list(false),
            BlockSyntax::Quote => self.toggle_quote(),
            BlockSyntax::Rule => self.insert_rule(),
            BlockSyntax::Fence(_, language) => self.set_kind(at.block, BlockKind::code(language)),
            BlockSyntax::Ordered(start, _) => {
                self.toggle_list(true);
                if let Some(list) =
                    self.nearest(at.block, |kind| matches!(kind, BlockKind::List { .. }))
                {
                    self.doc.get_mut(list).expect("the list").kind = BlockKind::ordered_list(start);
                }
                true
            }
        }
    }

    /// Enter at the end of a paragraph that is a whole fence, "```rust": a code block in
    /// that language, as a typed space after the fence makes one.
    pub(super) fn fence_rule(&mut self) -> bool {
        let at = self.caret();
        let block = self.block(at.block);
        if block.kind != BlockKind::Paragraph || at.offset != block.len() {
            return false;
        }
        let [Inline::Text { text, .. }] = block.inlines() else {
            return false;
        };
        let Some(BlockSyntax::Fence(_, language)) = block_syntax(text.trim_end()) else {
            return false;
        };
        self.delete_range(Position::new(at.block, 0), at);
        self.set_kind(at.block, BlockKind::code(language))
    }

    /// A closing delimiter typed after its opener: `` `x` ``, `**x**`, `*x*`, `~~x~~`.
    fn inline_rule(&mut self) -> bool {
        let at = self.caret();
        let text: Vec<char> = self
            .block(at.block)
            .text()
            .chars()
            .take(at.offset)
            .collect();
        let Some((open, len, mark)) = closing_span(&text, |_| true, &DELIMITERS) else {
            return false;
        };
        let kind = mark.kind();
        let block = at.block;
        let close = text.len() - len;
        self.delete_range(Position::new(block, close), Position::new(block, at.offset));
        self.delete_range(Position::new(block, open), Position::new(block, open + len));
        let end = close - len;
        let inlines = self.doc.get_mut(block).expect("a leaf").inlines_mut();
        super::doc::map_marks(inlines, open, end, |marks| marks.add(mark.clone()));
        self.set_caret(Position::new(block, end));
        let mut marks = self.current_marks();
        marks.remove(kind);
        self.stored_marks = Some(marks);
        true
    }
}
