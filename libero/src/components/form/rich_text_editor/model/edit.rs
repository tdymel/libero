//! Text edits on [`EditorState`]: typing, deleting, splitting and joining blocks, marks.
//! Each returns whether it changed anything.

use super::doc::{
    Block, BlockKind, Content, ContentKind, Inline, NodeKey, map_marks, mark_range,
    normalize_inlines, split_inlines,
};
use super::mark::{Href, Mark, MarkKind, UnsafeHref};
use super::state::{EditorState, Position, Selection};

impl EditorState {
    pub fn set_caret(&mut self, at: Position) {
        self.selection = Selection::caret(at);
        self.stored_marks = None;
        self.clamp();
    }

    pub fn select(&mut self, selection: Selection) {
        self.selection = selection;
        self.stored_marks = None;
        self.clamp();
    }

    pub fn select_all(&mut self) -> bool {
        let last = self.doc.last_leaf();
        let len = self.block(last).len();
        self.select(Selection::range(
            Position::new(self.doc.first_leaf(), 0),
            Position::new(last, len),
        ));
        true
    }

    /// Deletes the selected content. `false` for a collapsed selection.
    pub fn delete_selection(&mut self) -> bool {
        let (from, to) = self.ordered();
        self.delete_range(from, to)
    }

    /// Deletes between two ordered positions, joining the two end leaves.
    pub fn delete_range(&mut self, from: Position, to: Position) -> bool {
        if from == to {
            return false;
        }
        let leaves = self.doc.leaves();
        let last = *leaves.last().expect("a doc has a leaf");
        if from.block != to.block
            && from == Position::new(leaves[0], 0)
            && to == Position::new(last, self.block(last).len())
        {
            // Everything goes: start over from a plain paragraph, as other editors do.
            let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
            let key = paragraph.key;
            self.doc.blocks = vec![paragraph];
            self.set_caret(Position::new(key, 0));
            return true;
        }
        let (start, end) = (self.leaf_index(from.block), self.leaf_index(to.block));
        let before = start.checked_sub(1).map(|index| leaves[index]);
        let after = leaves.get(end + 1).copied();
        let a_inline = self.block(from.block).kind.content() == ContentKind::Inline;
        let b_inline = self.block(to.block).kind.content() == ContentKind::Inline;

        if from.block == to.block {
            match a_inline {
                true => {
                    let block = self.doc.get_mut(from.block).expect("a leaf");
                    let (head, rest) = split_inlines(block.inlines(), from.offset);
                    let (_, tail) = split_inlines(&rest, to.offset - from.offset);
                    *block.inlines_mut() = head.into_iter().chain(tail).collect();
                    normalize_inlines(block.inlines_mut());
                    self.set_caret(from);
                }
                false => {
                    self.remove(from.block);
                    self.settle(before, after);
                }
            }
            return true;
        }

        for key in &leaves[start + 1..end] {
            self.remove(*key);
        }
        let tail = match b_inline {
            true => {
                let block = self.block(to.block);
                Some(split_inlines(block.inlines(), to.offset).1)
            }
            false => None,
        };
        let keep_a = a_inline || from.offset == 1;
        let keep_b = !b_inline && to.offset == 0;
        if a_inline {
            let block = self.doc.get_mut(from.block).expect("a leaf");
            *block.inlines_mut() = split_inlines(block.inlines(), from.offset).0;
        } else if !keep_a {
            self.remove(from.block);
        }
        match (a_inline, tail) {
            (true, Some(tail)) => {
                self.remove(to.block);
                let kind = self.block(from.block).kind.clone();
                let block = self.doc.get_mut(from.block).expect("a leaf");
                block.inlines_mut().extend(coerce(&kind, tail));
                normalize_inlines(block.inlines_mut());
            }
            (false, Some(tail)) => {
                let block = self.doc.get_mut(to.block).expect("a leaf");
                *block.inlines_mut() = tail;
            }
            (_, None) if !keep_b => {
                self.remove(to.block);
            }
            _ => {}
        }
        self.prune();
        match (keep_a, a_inline) {
            (true, true) => self.set_caret(from),
            (true, false) => self.set_caret(Position::new(from.block, 1)),
            _ if self.doc.get(to.block).is_some() => self.set_caret(Position::new(to.block, 0)),
            _ => self.settle(before, after),
        }
        true
    }

    /// Types `text` at the caret, replacing the selection. `\n` splits blocks, except in code.
    pub fn insert_text(&mut self, text: &str) -> bool {
        self.delete_selection();
        if self.block_kind().is_code() {
            return self.insert_inline(Inline::text(text));
        }
        for (index, line) in text.split('\n').enumerate() {
            if index > 0 {
                self.split_block();
            }
            if !line.is_empty() {
                self.insert_inline(Inline::text(line));
            }
        }
        !text.is_empty()
    }

    /// Inserts one inline at the caret. Unmarked text takes the current marks.
    pub fn insert_inline(&mut self, inline: Inline) -> bool {
        self.delete_selection();
        let at = self.caret();
        if self.block(at.block).kind.content() != ContentKind::Inline {
            let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
            let key = paragraph.key;
            self.insert_after(at.block, at.offset == 0, paragraph);
            self.set_caret(Position::new(key, 0));
            return self.insert_inline(inline);
        }
        let code = self.block_kind().is_code();
        let inline = match inline {
            Inline::Text { text, .. } if code => Inline::text(text),
            Inline::Text { text, marks } if marks.is_empty() => {
                Inline::marked(text, self.current_marks())
            }
            Inline::HardBreak if code => Inline::text("\n"),
            Inline::Node { .. } if code => return false,
            other => other,
        };
        let len = inline.len();
        let block = self.doc.get_mut(at.block).expect("a leaf");
        let (head, tail) = split_inlines(block.inlines(), at.offset);
        *block.inlines_mut() = head.into_iter().chain([inline]).chain(tail).collect();
        normalize_inlines(block.inlines_mut());
        self.set_caret(Position::new(at.block, at.offset + len));
        true
    }

    pub fn insert_hard_break(&mut self) -> bool {
        self.insert_inline(Inline::HardBreak)
    }

    /// Enter: splits the leaf at the caret. In a list item the split makes a new item;
    /// in an empty item or at the end of a quote it leaves the container instead.
    pub fn split_block(&mut self) -> bool {
        self.delete_selection();
        let at = self.caret();
        let block = self.block(at.block).clone();
        if block.kind.content() == ContentKind::Atom {
            let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
            let key = paragraph.key;
            self.insert_after(at.block, at.offset == 0, paragraph);
            if at.offset == 1 {
                self.set_caret(Position::new(key, 0));
            }
            return true;
        }
        if block.kind.is_code() {
            return self.insert_inline(Inline::text("\n"));
        }
        let parent = self.parent(at.block);
        if block.is_empty() {
            match parent.as_ref().map(|(_, kind, _)| kind) {
                Some(BlockKind::ListItem { .. }) => return self.outdent(),
                Some(BlockKind::Quote) if self.is_last_child(at.block) => {
                    return self.lift_out_of_quote(at.block);
                }
                _ => {}
            }
        }
        let (head, tail) = split_inlines(block.inlines(), at.offset);
        let kind = match block.kind {
            BlockKind::Heading { .. } if tail.is_empty() => BlockKind::Paragraph,
            kind => kind,
        };
        *self.doc.get_mut(at.block).expect("a leaf").inlines_mut() = head;
        let new = self.doc.leaf(kind, tail);
        let key = new.key;
        match parent {
            Some((item, BlockKind::ListItem { checked }, index)) => {
                let item_block = self.doc.get_mut(item).expect("the item");
                let rest = item_block.children_mut().split_off(index + 1);
                let children = std::iter::once(new).chain(rest).collect();
                // The item after a task is an open task.
                let kind = BlockKind::ListItem {
                    checked: checked.map(|_| false),
                };
                let new_item = self.doc.container(kind, children);
                self.insert_after(item, false, new_item);
            }
            _ => self.insert_after(at.block, false, new),
        }
        self.set_caret(Position::new(key, 0));
        true
    }

    /// Backspace.
    pub fn delete_backward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let at = self.caret();
        let block = self.block(at.block);
        if block.kind.content() == ContentKind::Atom && at.offset == 1 {
            return self.delete_range(Position::new(at.block, 0), at);
        }
        if at.offset > 0 {
            let back = grapheme_back(&block.text(), at.offset);
            return self.delete_range(Position::new(at.block, at.offset - back), at);
        }
        let is_paragraph = block.kind == BlockKind::Paragraph;
        match self.parent(at.block) {
            Some((_, BlockKind::ListItem { .. }, 0)) => return self.outdent(),
            Some((_, BlockKind::Quote, 0)) => return self.lift_out_of_quote(at.block),
            _ => {}
        }
        let index = self.leaf_index(at.block);
        let Some(previous) = index.checked_sub(1).map(|index| self.doc.leaves()[index]) else {
            return !is_paragraph && self.set_kind(at.block, BlockKind::Paragraph);
        };
        let previous_len = self.block(previous).len();
        match self.block(previous).kind.content() {
            ContentKind::Atom => {
                self.delete_range(Position::new(previous, 0), Position::new(previous, 1))
            }
            _ => self.delete_range(Position::new(previous, previous_len), at),
        };
        true
    }

    /// Delete.
    pub fn delete_forward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let at = self.caret();
        let block = self.block(at.block);
        let len = block.len();
        if block.kind.content() == ContentKind::Atom && at.offset == 0 {
            return self.delete_range(at, Position::new(at.block, 1));
        }
        if at.offset < len {
            let forward = grapheme_forward(&block.text(), at.offset);
            return self.delete_range(at, Position::new(at.block, at.offset + forward));
        }
        let leaves = self.doc.leaves();
        let Some(next) = leaves.get(self.leaf_index(at.block) + 1).copied() else {
            return false;
        };
        match self.block(next).kind.content() {
            ContentKind::Atom => {
                self.remove(next);
                self.prune();
                self.set_caret(at);
                true
            }
            _ => self.delete_range(at, Position::new(next, 0)),
        }
    }

    /// Ctrl/Alt+Backspace: back to the previous word start, or one step at a block start.
    pub(crate) fn delete_word_backward(&mut self) -> bool {
        self.delete_back_to(word_back)
    }

    /// Ctrl/Alt+Delete: on to the next word end, or one step at a block end.
    pub(crate) fn delete_word_forward(&mut self) -> bool {
        self.delete_on_to(word_forward)
    }

    /// Cmd+Backspace: back to the line start (a hard break or the block start).
    pub(crate) fn delete_line_backward(&mut self) -> bool {
        self.delete_back_to(|chars: &[char], offset| {
            chars[..offset]
                .iter()
                .rposition(|&c| c == '\n')
                .map_or(0, |at| at + 1)
        })
    }

    /// Cmd+Delete: on to the line end (a hard break or the block end).
    pub(crate) fn delete_line_forward(&mut self) -> bool {
        self.delete_on_to(|chars: &[char], offset| {
            chars[offset..]
                .iter()
                .position(|&c| c == '\n')
                .map_or(chars.len(), |at| offset + at)
        })
    }

    /// Deletes from `start(chars, caret)` to the caret; at a block start or on an atom, Backspace.
    fn delete_back_to(&mut self, start: impl Fn(&[char], usize) -> usize) -> bool {
        if self.delete_selection() {
            return true;
        }
        let at = self.caret();
        let block = self.block(at.block);
        if at.offset == 0 || block.kind.content() == ContentKind::Atom {
            return self.delete_backward();
        }
        let chars: Vec<char> = block.text().chars().collect();
        let from = start(&chars, at.offset).min(at.offset - 1);
        self.delete_range(Position::new(at.block, from), at)
    }

    /// Deletes from the caret to `end(chars, caret)`; at a block end or on an atom, Delete.
    fn delete_on_to(&mut self, end: impl Fn(&[char], usize) -> usize) -> bool {
        if self.delete_selection() {
            return true;
        }
        let at = self.caret();
        let block = self.block(at.block);
        if at.offset >= block.len() || block.kind.content() == ContentKind::Atom {
            return self.delete_forward();
        }
        let chars: Vec<char> = block.text().chars().collect();
        let to = end(&chars, at.offset).max(at.offset + 1);
        self.delete_range(at, Position::new(at.block, to))
    }

    /// Adds `mark` to the selection, or removes its kind when all of it has one.
    /// At a collapsed caret it toggles the marks the next typed text takes.
    pub fn toggle_mark(&mut self, mark: Mark) -> bool {
        let kind = mark.kind();
        if self.selection.is_collapsed() {
            if self.block_kind().is_code() || self.block_kind().content() != ContentKind::Inline {
                return false;
            }
            let mut marks = self.current_marks();
            match marks.has(kind) {
                true => marks.remove(kind),
                false => marks.add(mark),
            }
            self.stored_marks = Some(marks);
            return true;
        }
        let remove = self.is_active(kind);
        self.change_marks(|marks| match remove {
            true => marks.remove(kind),
            false => marks.add(mark.clone()),
        })
    }

    fn change_marks(&mut self, change: impl Fn(&mut super::mark::Marks)) -> bool {
        let segments = self.text_segments();
        let mut changed = false;
        for (key, lo, hi) in segments {
            if lo == hi {
                continue;
            }
            let block = self.doc.get_mut(key).expect("a leaf");
            let before = block.inlines().to_vec();
            map_marks(block.inlines_mut(), lo, hi, &change);
            changed |= before != block.inlines();
        }
        changed
    }

    /// Links the selection to `href`. At a caret inside a link it changes that
    /// link's href; elsewhere it inserts the href as linked text.
    pub fn set_link(&mut self, href: &str) -> Result<bool, UnsafeHref> {
        let href = Href::parse(href)?;
        let mark = Mark::Link {
            href: href.clone(),
            title: None,
        };
        if !self.selection.is_collapsed() {
            return Ok(self.change_marks(|marks| marks.add(mark.clone())));
        }
        if let Some((key, from, to)) = self.link_at_caret() {
            let block = self.doc.get_mut(key).expect("a leaf");
            map_marks(block.inlines_mut(), from, to, |marks| {
                marks.add(mark.clone())
            });
            return Ok(true);
        }
        let marks = self.current_marks().with(mark);
        Ok(self.insert_inline(Inline::marked(href.as_str(), marks)))
    }

    /// Removes links from the selection, or the whole link around the caret.
    pub fn remove_link(&mut self) -> bool {
        if !self.selection.is_collapsed() {
            return self.change_marks(|marks| marks.remove(MarkKind::Link));
        }
        let Some((key, from, to)) = self.link_at_caret() else {
            return false;
        };
        let block = self.doc.get_mut(key).expect("a leaf");
        map_marks(block.inlines_mut(), from, to, |marks| {
            marks.remove(MarkKind::Link)
        });
        true
    }

    /// The link run touching the caret.
    pub fn link_at_caret(&self) -> Option<(NodeKey, usize, usize)> {
        let at = self.caret();
        let block = self.block(at.block);
        let inlines = block.inlines();
        let probe = |offset: usize| {
            let marks = super::doc::slice_inlines(inlines, offset, offset + 1);
            marks.iter().find_map(|inline| match inline {
                Inline::Text { marks, .. } => marks.get(MarkKind::Link).cloned(),
                _ => None,
            })
        };
        let link = probe(at.offset).or_else(|| at.offset.checked_sub(1).and_then(probe))?;
        let (from, to) = mark_range(inlines, at.offset, &link)?;
        Some((at.block, from, to))
    }

    /// Sets the kind of every inline leaf in the selection.
    pub fn set_text_kind(&mut self, kind: BlockKind) -> bool {
        let keys: Vec<NodeKey> = self.segments().iter().map(|(key, ..)| *key).collect();
        let mut changed = false;
        for key in keys {
            if self.block(key).kind.content() == ContentKind::Inline {
                changed |= self.set_kind(key, kind.clone());
            }
        }
        changed
    }

    /// Heading `level`, or back to a paragraph when every selected leaf is one already.
    pub fn toggle_heading(&mut self, level: u8) -> bool {
        let heading = BlockKind::heading(level);
        self.toggle_text_kind(heading)
    }

    pub fn toggle_code_block(&mut self) -> bool {
        let all_code = self.selected_leaf_kinds().iter().all(BlockKind::is_code);
        match all_code {
            true => self.set_text_kind(BlockKind::Paragraph),
            false => self.set_text_kind(BlockKind::code("")),
        }
    }

    /// The language of the code block at the caret; `""` for none.
    pub(crate) fn set_code_language(&mut self, language: &str) -> bool {
        let key = self.caret().block;
        self.block(key).kind.is_code() && self.set_kind(key, BlockKind::code(language))
    }

    fn toggle_text_kind(&mut self, kind: BlockKind) -> bool {
        let all = self
            .selected_leaf_kinds()
            .iter()
            .all(|other| *other == kind);
        match all {
            true => self.set_text_kind(BlockKind::Paragraph),
            false => self.set_text_kind(kind),
        }
    }

    fn selected_leaf_kinds(&self) -> Vec<BlockKind> {
        self.segments()
            .iter()
            .map(|(key, ..)| self.block(*key).kind.clone())
            .filter(|kind| kind.content() == ContentKind::Inline)
            .collect()
    }

    pub(super) fn set_kind(&mut self, key: NodeKey, kind: BlockKind) -> bool {
        let block = self.doc.get_mut(key).expect("a leaf");
        if block.kind == kind {
            return false;
        }
        *block.inlines_mut() = coerce(&kind, std::mem::take(block.inlines_mut()));
        block.kind = kind;
        true
    }

    /// Inserts a block (a rule, a caller's atom) at the caret, splitting the leaf
    /// when the caret is inside it. The caret goes to the next text leaf.
    pub fn insert_block(&mut self, block: Block) -> bool {
        self.delete_selection();
        let at = self.caret();
        let current = self.block(at.block);
        let len = current.len();
        let empty_text = current.kind.content() == ContentKind::Inline && len == 0;
        let target = match (at.offset, empty_text) {
            (_, true) => at.block,
            (0, false) => {
                let key = block.key;
                self.insert_after(at.block, true, block);
                self.set_caret(Position::new(key, 1));
                return true;
            }
            (offset, false) if offset < len && current.kind.content() == ContentKind::Inline => {
                self.split_block();
                let key = self.caret().block;
                self.insert_after(key, true, block);
                self.set_caret(Position::new(key, 0));
                return true;
            }
            _ => at.block,
        };
        let key = block.key;
        let leaf = block.is_leaf() && block.kind.content() == ContentKind::Inline;
        self.insert_after(target, false, block);
        if empty_text {
            self.remove(target);
        }
        if leaf {
            self.set_caret(Position::new(key, 0));
            return true;
        }
        let leaves = self.doc.leaves();
        let next = leaves.get(self.leaf_index(key) + 1).copied();
        match next.filter(|next| self.block(*next).kind.content() == ContentKind::Inline) {
            Some(next) => self.set_caret(Position::new(next, 0)),
            None => {
                let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
                let paragraph_key = paragraph.key;
                self.insert_after(key, false, paragraph);
                self.set_caret(Position::new(paragraph_key, 0));
            }
        }
        true
    }

    pub fn insert_rule(&mut self) -> bool {
        let rule = self.doc.leaf(BlockKind::Rule, Vec::new());
        self.insert_block(rule)
    }

    /// Leaves the code block at the caret for a new paragraph right after it.
    pub fn exit_code(&mut self) -> bool {
        let at = self.caret();
        if !self.block(at.block).kind.is_code() {
            return false;
        }
        let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
        let key = paragraph.key;
        self.insert_after(at.block, false, paragraph);
        self.set_caret(Position::new(key, 0));
        true
    }

    /// Enter on the empty last line of a code block: drops that line and leaves the
    /// block, as Mod+Enter does. Touch keyboards have no other way out.
    pub(crate) fn exit_code_on_blank_end(&mut self) -> bool {
        let at = self.caret();
        let block = self.block(at.block);
        if !block.kind.is_code() || at.offset == 0 || at.offset != block.len() {
            return false;
        }
        if !block.text().ends_with('\n') {
            return false;
        }
        self.delete_range(Position::new(at.block, at.offset - 1), at);
        self.exit_code()
    }

    /// The caret into a text line after the last block: a new paragraph when the doc
    /// ends in a code block, a rule or an atom, which have no line to click into.
    pub(crate) fn exit_end(&mut self) -> bool {
        let Some(last) = self.doc.leaves().last().copied() else {
            return false;
        };
        let block = self.block(last);
        if block.kind.content() == ContentKind::Inline && !block.kind.is_code() {
            self.set_caret(Position::new(last, block.len()));
            return true;
        }
        let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
        let key = paragraph.key;
        self.doc.blocks.push(paragraph);
        self.set_caret(Position::new(key, 0));
        true
    }

    /// The container holding `key`, its kind and `key`'s index in it.
    pub(super) fn parent(&self, key: NodeKey) -> Option<(NodeKey, BlockKind, usize)> {
        let path = self.doc.path(key)?;
        let (index, parent) = path.split_last()?;
        if parent.is_empty() {
            return None;
        }
        let block = self.doc.at(parent);
        Some((block.key, block.kind.clone(), *index))
    }

    pub(super) fn is_last_child(&self, key: NodeKey) -> bool {
        let path = self.doc.path(key).expect("a key from this doc");
        let (index, parent) = path.split_last().expect("a path");
        let count = match parent {
            [] => self.doc.blocks.len(),
            parent => self.doc.at(parent).children().len(),
        };
        index + 1 == count
    }

    /// Inserts `block` next to `key`, before it when `before`.
    pub(super) fn insert_after(&mut self, key: NodeKey, before: bool, block: Block) {
        let path = self.doc.path(key).expect("a key from this doc");
        let index = *path.last().expect("a path") + usize::from(!before);
        self.doc.siblings_mut(&path).insert(index, block);
    }

    pub(super) fn remove(&mut self, key: NodeKey) -> Option<Block> {
        let path = self.doc.path(key)?;
        Some(self.doc.siblings_mut(&path).remove(*path.last()?))
    }

    /// Drops emptied containers and makes sure a leaf is left.
    pub(super) fn prune(&mut self) {
        fn prune(blocks: &mut Vec<Block>) {
            for block in blocks.iter_mut() {
                if let Content::Blocks(children) = &mut block.content {
                    prune(children);
                }
            }
            blocks.retain(|block| block.is_leaf() || !block.children().is_empty());
        }
        prune(&mut self.doc.blocks);
        if self.doc.blocks.is_empty() {
            let paragraph = self.doc.leaf(BlockKind::Paragraph, Vec::new());
            self.doc.blocks.push(paragraph);
        }
    }

    /// Puts the caret back after a whole-leaf removal: the next leaf, else the previous one.
    fn settle(&mut self, before: Option<NodeKey>, after: Option<NodeKey>) {
        self.prune();
        let exists = |key: &NodeKey| self.doc.get(*key).is_some();
        match (after.filter(exists), before.filter(exists)) {
            (Some(after), _) => self.set_caret(Position::new(after, 0)),
            (None, Some(before)) => {
                let len = self.block(before).len();
                self.set_caret(Position::new(before, len));
            }
            _ => {
                let first = self.doc.first_leaf();
                self.set_caret(Position::new(first, 0));
            }
        }
    }
}

/// Fits inlines to a block kind: code keeps only plain text, hard breaks become `\n`
/// and back.
pub(super) fn coerce(kind: &BlockKind, inlines: Vec<Inline>) -> Vec<Inline> {
    let mut out = Vec::with_capacity(inlines.len());
    for inline in inlines {
        match (kind.is_code(), inline) {
            (true, Inline::Text { text, .. }) => out.push(Inline::text(text)),
            (true, Inline::HardBreak) => out.push(Inline::text("\n")),
            (true, Inline::Node { .. }) => {}
            (false, Inline::Text { text, marks }) if text.contains('\n') => {
                for (index, line) in text.split('\n').enumerate() {
                    if index > 0 {
                        out.push(Inline::HardBreak);
                    }
                    out.push(Inline::marked(line, marks.clone()));
                }
            }
            (false, inline) => out.push(inline),
        }
    }
    normalize_inlines(&mut out);
    out
}

/// Chars that attach to the one before: joiners, variation selectors, skin tones, combining marks.
fn extends(c: char) -> bool {
    matches!(c as u32,
        0x200D | 0xFE0E | 0xFE0F | 0x1F3FB..=0x1F3FF | 0x0300..=0x036F | 0x20D0..=0x20FF | 0xE0020..=0xE007F)
}

fn regional(c: char) -> bool {
    matches!(c as u32, 0x1F1E6..=0x1F1FF)
}

/// Chars from `offset` back to the previous user-perceived character start (approximate).
pub(super) fn grapheme_back(text: &str, offset: usize) -> usize {
    let chars: Vec<char> = text.chars().take(offset).collect();
    let mut start = chars.len().saturating_sub(1);
    while start > 0 && (extends(chars[start]) || chars[start - 1] == '\u{200D}') {
        start -= 1;
    }
    if start > 0 && regional(chars[start]) {
        let run = chars[..=start]
            .iter()
            .rev()
            .take_while(|c| regional(**c))
            .count();
        if run % 2 == 0 {
            start -= 1;
        }
    }
    offset - start
}

/// Chars from `offset` forward to the next user-perceived character start (approximate).
pub(super) fn grapheme_forward(text: &str, offset: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut end = offset + 1;
    if regional(chars[offset]) && chars.get(end).copied().is_some_and(regional) {
        let run = chars[..offset]
            .iter()
            .rev()
            .take_while(|c| regional(**c))
            .count();
        if run % 2 == 0 {
            end += 1;
        }
    }
    while end < chars.len() && (extends(chars[end]) || chars[end - 1] == '\u{200D}') {
        end += 1;
    }
    end - offset
}

/// Letters, digits and `_` form words; any other non-space run (`...`, an emoji) is one too.
fn word_class(c: char) -> u8 {
    match c {
        _ if c.is_whitespace() => 0,
        _ if c.is_alphanumeric() || c == '_' => 1,
        _ => 2,
    }
}

/// The start of the word before `offset`, skipping the spaces in between.
fn word_back(chars: &[char], offset: usize) -> usize {
    let mut start = offset;
    while start > 0 && word_class(chars[start - 1]) == 0 && chars[start - 1] != '\n' {
        start -= 1;
    }
    // Joiners and combining marks go with their base, whichever class it is.
    let class = chars[..start]
        .iter()
        .rev()
        .find(|&&c| !extends(c))
        .map_or(0, |&c| word_class(c));
    while start > 0
        && class != 0
        && (extends(chars[start - 1]) || word_class(chars[start - 1]) == class)
    {
        start -= 1;
    }
    start
}

/// The end of the word after `offset`, skipping the spaces in between.
fn word_forward(chars: &[char], offset: usize) -> usize {
    let mut end = offset;
    while end < chars.len() && word_class(chars[end]) == 0 && chars[end] != '\n' {
        end += 1;
    }
    let class = chars[end..]
        .iter()
        .find(|&&c| !extends(c))
        .map_or(0, |&c| word_class(c));
    while end < chars.len()
        && class != 0
        && (extends(chars[end]) || word_class(chars[end]) == class)
    {
        end += 1;
    }
    end
}
