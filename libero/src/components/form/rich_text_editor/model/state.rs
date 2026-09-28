//! [`EditorState`]: a doc, a selection and the marks the next typed text takes.

use serde::{Deserialize, Serialize};

use super::doc::{
    Block, BlockKind, Content, ContentKind, Doc, NodeKey, all_marked, marks_at, slice_inlines,
};
use super::mark::{MarkKind, Marks};

/// A caret spot: a char offset into a leaf. An atom leaf has offsets 0 (before) and 1 (after).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub block: NodeKey,
    pub offset: usize,
}

impl Position {
    pub fn new(block: NodeKey, offset: usize) -> Self {
        Self { block, offset }
    }
}

/// `anchor` stays put while `head` moves; either may come first in the doc.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Selection {
    pub anchor: Position,
    pub head: Position,
}

impl Selection {
    pub fn caret(at: Position) -> Self {
        Self {
            anchor: at,
            head: at,
        }
    }

    pub fn range(anchor: Position, head: Position) -> Self {
        Self { anchor, head }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorState {
    pub doc: Doc,
    pub selection: Selection,
    /// Marks toggled at a collapsed caret, taken by the next typed text.
    pub stored_marks: Option<Marks>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new(Doc::new())
    }
}

impl EditorState {
    /// The caret at the start of `doc`.
    pub fn new(doc: Doc) -> Self {
        let first = doc.first_leaf();
        Self {
            doc,
            selection: Selection::caret(Position::new(first, 0)),
            stored_marks: None,
        }
    }

    pub fn with_selection(mut self, selection: Selection) -> Self {
        self.selection = selection;
        self.clamp();
        self
    }

    pub fn caret(&self) -> Position {
        self.selection.head
    }

    /// Moves a selection that points at a removed block or past its end back into the doc.
    pub fn clamp(&mut self) {
        let fix = |doc: &Doc, at: Position| match doc.get(at.block).filter(|b| b.is_leaf()) {
            Some(block) => Position::new(at.block, at.offset.min(block.len())),
            None => Position::new(doc.first_leaf(), 0),
        };
        self.selection.anchor = fix(&self.doc, self.selection.anchor);
        self.selection.head = fix(&self.doc, self.selection.head);
    }

    pub fn leaf_index(&self, key: NodeKey) -> usize {
        self.doc
            .leaves()
            .iter()
            .position(|leaf| *leaf == key)
            .unwrap_or(0)
    }

    /// The selection's ends in reading order.
    pub fn ordered(&self) -> (Position, Position) {
        let Selection { anchor, head } = self.selection;
        let order = (self.leaf_index(anchor.block), anchor.offset)
            .cmp(&(self.leaf_index(head.block), head.offset));
        match order {
            std::cmp::Ordering::Greater => (head, anchor),
            _ => (anchor, head),
        }
    }

    pub fn block(&self, key: NodeKey) -> &Block {
        self.doc.get(key).expect("a key from this doc")
    }

    /// Every leaf the selection touches with the char range it covers in each.
    pub fn segments(&self) -> Vec<(NodeKey, usize, usize)> {
        let (from, to) = self.ordered();
        let leaves = self.doc.leaves();
        let (start, end) = (self.leaf_index(from.block), self.leaf_index(to.block));
        leaves[start..=end]
            .iter()
            .map(|key| {
                let len = self.block(*key).len();
                let lo = if *key == from.block { from.offset } else { 0 };
                let hi = if *key == to.block { to.offset } else { len };
                (*key, lo, hi)
            })
            .collect()
    }

    /// Text segments of the selection, code blocks left out: where marks apply.
    pub fn text_segments(&self) -> Vec<(NodeKey, usize, usize)> {
        self.segments()
            .into_iter()
            .filter(|(key, ..)| {
                let block = self.block(*key);
                block.kind.content() == ContentKind::Inline && !block.kind.is_code()
            })
            .collect()
    }

    /// The marks typing would take now.
    pub fn current_marks(&self) -> Marks {
        if let Some(stored) = &self.stored_marks {
            return stored.clone();
        }
        let at = self.caret();
        let block = self.block(at.block);
        match block.kind.content() == ContentKind::Inline && !block.kind.is_code() {
            true => marks_at(block.inlines(), at.offset),
            false => Marks::new(),
        }
    }

    /// Whether a mark of `kind` is on: on the whole selection, or at the caret.
    pub fn is_active(&self, kind: MarkKind) -> bool {
        if self.selection.is_collapsed() {
            return self.current_marks().has(kind);
        }
        let segments: Vec<_> = self
            .text_segments()
            .into_iter()
            .filter(|(key, lo, hi)| {
                lo < hi && !slice_inlines(self.block(*key).inlines(), *lo, *hi).is_empty()
            })
            .collect();
        !segments.is_empty()
            && segments
                .iter()
                .all(|(key, lo, hi)| all_marked(self.block(*key).inlines(), *lo, *hi, kind))
    }

    /// The kind of the leaf holding the caret.
    pub fn block_kind(&self) -> &BlockKind {
        &self.block(self.caret().block).kind
    }

    /// The kinds of the containers around `key`, innermost first.
    pub fn ancestors(&self, key: NodeKey) -> Vec<(NodeKey, BlockKind)> {
        let path = self.doc.path(key).unwrap_or_default();
        (1..path.len())
            .rev()
            .map(|len| {
                let block = self.doc.at(&path[..len]);
                (block.key, block.kind.clone())
            })
            .collect()
    }

    /// The nearest list around the caret: `Some(ordered)`.
    pub fn list_kind(&self) -> Option<bool> {
        self.ancestors(self.caret().block)
            .into_iter()
            .find_map(|(_, kind)| match kind {
                BlockKind::List { ordered, .. } => Some(ordered),
                _ => None,
            })
    }

    pub fn in_quote(&self) -> bool {
        self.ancestors(self.caret().block)
            .iter()
            .any(|(_, kind)| *kind == BlockKind::Quote)
    }

    /// The caret block's text up to the caret, an inline node as U+FFFC: what a trigger
    /// such as `@` for mentions looks at.
    pub fn text_before_caret(&self) -> String {
        let at = self.caret();
        self.block(at.block)
            .text()
            .chars()
            .take(at.offset)
            .collect()
    }

    /// The selected text, leaves joined by `\n`.
    pub fn selected_text(&self) -> String {
        self.segments()
            .iter()
            .map(|(key, lo, hi)| {
                let text = self.block(*key).text();
                text.chars().skip(*lo).take(hi - lo).collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The selection as a doc of its own: leaves cut to it, their containers kept.
    pub(crate) fn selected_doc(&self) -> Doc {
        fn cut(blocks: &[Block], ranges: &[(NodeKey, usize, usize)]) -> Vec<Block> {
            let range = |key| {
                ranges
                    .iter()
                    .find(|(k, ..)| *k == key)
                    .map(|&(_, lo, hi)| (lo, hi))
            };
            blocks
                .iter()
                .filter_map(|block| {
                    let content = match &block.content {
                        Content::Blocks(children) => {
                            let kept = cut(children, ranges);
                            (!kept.is_empty()).then_some(Content::Blocks(kept))
                        }
                        Content::Inlines(inlines) => range(block.key)
                            .map(|(lo, hi)| Content::Inlines(slice_inlines(inlines, lo, hi))),
                        Content::Empty => range(block.key)
                            .filter(|(lo, hi)| lo < hi)
                            .map(|_| Content::Empty),
                    }?;
                    Some(Block {
                        key: block.key,
                        kind: block.kind.clone(),
                        content,
                    })
                })
                .collect()
        }
        let mut doc = Doc::empty();
        doc.blocks = cut(&self.doc.blocks, &self.segments());
        doc
    }
}
