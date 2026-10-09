//! Markdown shortcuts: typed syntax turns into structure (`# ` a heading, `**x**` bold).
//! The recognizers live in [`super::syntax`], shared with the Markdown reader.

use super::doc::{
    BlockKind, ContentKind, Inline, NodeKey, map_marks, mark_range, normalize_inlines,
    slice_inlines, split_inlines,
};
use super::mark::{Mark, MarkKind};
use super::markdown::closer;
use super::state::{EditorState, Position};
use super::syntax::{
    BlockSyntax, DELIMITERS, autolink, block_syntax, closing_span, link, task_box,
};

/// Where the whitespace-free word ending `text` starts.
fn word_start(text: &[char]) -> usize {
    text.iter()
        .rposition(|c| c.is_whitespace() || *c == '\u{fffc}')
        .map_or(0, |at| at + 1)
}

fn splice_text(inlines: &mut Vec<Inline>, at: usize, text: &str) {
    let (before, after) = split_inlines(inlines, at);
    *inlines = before
        .into_iter()
        .chain([Inline::text(text)])
        .chain(after)
        .collect();
    normalize_inlines(inlines);
}

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
        self.block_rule() || self.inline_rule() || self.link_rule() || self.autolink_rule()
    }

    /// A prefix typed at the start of a paragraph (or a task box in a list item), ended by a space.
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
        let Some(prefix) = text.strip_suffix(' ') else {
            return false;
        };
        if let Some(checked) = task_box(prefix) {
            return self.task_rule(checked);
        }
        let Some(syntax) = block_syntax(prefix) else {
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

    /// `[ ] ` or `[x] ` at the start of a list item makes it a task item.
    fn task_rule(&mut self, checked: bool) -> bool {
        let at = self.caret();
        let Some((item, BlockKind::ListItem { .. }, 0)) = self.parent(at.block) else {
            return false;
        };
        self.delete_range(Position::new(at.block, 0), at);
        self.doc.get_mut(item).expect("the item").kind = BlockKind::task_item(checked);
        true
    }

    /// Enter at the end of a paragraph that is a whole fence, "```rust": a code block in
    /// that language, as a typed space after the fence makes one.
    /// Whether Enter here turns a typed fence into a code block rather than splitting.
    pub(crate) fn enter_opens_fence(&self) -> bool {
        self.clone().fence_rule()
    }

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

    fn chars_before_caret(&self) -> Vec<char> {
        self.text_before_caret().chars().collect()
    }

    /// The link a typed `)` completes: `[text](url)` becomes `text` linked to `url`.
    fn link_rule(&mut self) -> bool {
        let at = self.caret();
        let text = self.chars_before_caret();
        if text.last() != Some(&')') {
            return false;
        }
        let found = (0..text.len())
            .rev()
            .filter(|open| text[*open] == '[' && (*open == 0 || text[*open - 1] != '!'))
            .find_map(|open| {
                link(&text, open)
                    .filter(|(close, end, _)| *end == text.len() && *close > open + 1)
                    .map(|(close, _, mark)| (open, close, mark))
            });
        let Some((open, close, mark)) = found else {
            return false;
        };
        let block = at.block;
        self.delete_range(Position::new(block, close), at);
        self.delete_range(Position::new(block, open), Position::new(block, open + 1));
        let end = close - 1;
        let inlines = self.doc.get_mut(block).expect("a leaf").inlines_mut();
        super::doc::map_marks(inlines, open, end, |marks| marks.add(mark.clone()));
        self.set_caret(Position::new(block, end));
        let mut marks = self.current_marks();
        marks.remove(MarkKind::Link);
        self.stored_marks = Some(marks);
        true
    }

    /// A URL followed by the space just typed becomes a link. Stored marks flag the
    /// moment for [`revert_link_rule`](Self::revert_link_rule); typing or moving drops them.
    fn autolink_rule(&mut self) -> bool {
        let linked = match self.chars_before_caret().split_last() {
            Some((' ', word)) => self.autolink(word.len()),
            _ => false,
        };
        if linked {
            self.stored_marks = Some(self.current_marks());
        }
        linked
    }

    /// Backspace right after a link rule fired: a typed `[text](url)` goes back to that
    /// text, an autolinked URL back to plain text (todo 2725).
    pub(super) fn revert_link_rule(&mut self) -> bool {
        let at = self.caret();
        let block = self.block(at.block);
        if self.stored_marks.is_none()
            || at.offset == 0
            || block.kind.content() != ContentKind::Inline
            || block.kind.is_code()
        {
            return false;
        }
        let last = slice_inlines(block.inlines(), at.offset - 1, at.offset);
        let link = last.iter().find_map(|inline| match inline {
            Inline::Text { marks, .. } => marks.iter().find(|m| m.kind() == MarkKind::Link),
            _ => None,
        });
        match link.cloned() {
            Some(mark) => self.revert_typed_link(at, &mark),
            None => self.revert_autolink(at.block, at.offset - 1),
        }
    }

    /// `text` linked ending at the caret goes back to `[text](url)`, the caret after it.
    fn revert_typed_link(&mut self, at: Position, mark: &Mark) -> bool {
        let inlines = self.block(at.block).inlines();
        let Some((start, end)) =
            mark_range(inlines, at.offset, mark).filter(|(_, end)| *end == at.offset)
        else {
            return false;
        };
        let closer = closer(mark, false);
        let inlines = self.doc.get_mut(at.block).expect("a leaf").inlines_mut();
        map_marks(inlines, start, end, |marks| marks.remove(MarkKind::Link));
        splice_text(inlines, end, &closer);
        splice_text(inlines, start, "[");
        self.set_caret(Position::new(at.block, end + 1 + closer.chars().count()));
        true
    }

    /// The linked URL before the space at `space` goes back to plain text, the space kept.
    fn revert_autolink(&mut self, block: NodeKey, space: usize) -> bool {
        let text: Vec<char> = self.block(block).text().chars().take(space + 1).collect();
        if text.last() != Some(&' ') {
            return false;
        }
        let word = &text[..space];
        let from = word_start(word);
        let Some((len, href)) = autolink(&word[from..]) else {
            return false;
        };
        let to = from + len;
        let linked = slice_inlines(self.block(block).inlines(), from, to)
            .iter()
            .all(|inline| {
                matches!(inline, Inline::Text { marks, .. }
                    if marks.iter().any(|m| matches!(m, Mark::Link { href: h, .. } if *h == href)))
            });
        if !linked {
            return false;
        }
        let inlines = self.doc.get_mut(block).expect("a leaf").inlines_mut();
        map_marks(inlines, from, to, |marks| marks.remove(MarkKind::Link));
        self.stored_marks = None;
        true
    }

    /// Enter after a URL links it before the block splits.
    pub(super) fn autolink_at_caret(&mut self) -> bool {
        let at = self.caret();
        let block = self.block(at.block);
        self.selection.is_collapsed()
            && block.kind.content() == ContentKind::Inline
            && !block.kind.is_code()
            && self.autolink(at.offset)
    }

    /// Links the URL that is the last word before `end` in the caret's block, unless
    /// it is code or already a link.
    fn autolink(&mut self, end: usize) -> bool {
        let block = self.caret().block;
        let text: Vec<char> = self.block(block).text().chars().take(end).collect();
        let from = word_start(&text);
        let Some((len, href)) = autolink(&text[from..]) else {
            return false;
        };
        let to = from + len;
        let plain = slice_inlines(self.block(block).inlines(), from, to)
            .iter()
            .all(|inline| {
                matches!(inline, Inline::Text { marks, .. }
                    if !marks.has(MarkKind::Link) && !marks.has(MarkKind::Code))
            });
        if !plain {
            return false;
        }
        let mark = Mark::Link { href, title: None };
        let inlines = self.doc.get_mut(block).expect("a leaf").inlines_mut();
        super::doc::map_marks(inlines, from, to, |marks| marks.add(mark.clone()));
        true
    }

    /// A closing delimiter typed after its opener: `` `x` ``, `**x**`, `*x*`, `~~x~~`.
    fn inline_rule(&mut self) -> bool {
        let at = self.caret();
        let text = self.chars_before_caret();
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
