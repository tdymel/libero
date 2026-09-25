//! The document tree: blocks with stable keys, holding blocks or inline content.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::mark::{Mark, MarkKind, Marks};

/// Identifies a block for its whole life: edits keep it, a split gives the new half a new one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeKey(pub u64);

/// Attributes of a caller-registered node.
pub type Attrs = BTreeMap<String, serde_json::Value>;

/// What a block holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentKind {
    /// Text and inline nodes; the caret lives here.
    Inline,
    /// Other blocks.
    Blocks,
    /// Nothing: a leaf selected as a whole, such as a rule.
    Atom,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum BlockKind {
    Paragraph,
    Heading {
        level: u8,
    },
    /// `language` is the raw fence info, kept even when no grammar knows it.
    CodeBlock {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        language: String,
    },
    Quote,
    List {
        ordered: bool,
        #[serde(default = "one", skip_serializing_if = "is_one")]
        start: u64,
    },
    ListItem,
    Rule,
    /// A caller's node, described by its `NodeSpec` in the registry.
    Custom {
        name: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        attrs: Attrs,
        content: CustomContent,
    },
}

/// The content of a custom block, stored on the node so a doc reads without its registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomContent {
    Inline,
    Blocks,
    Atom,
}

fn one() -> u64 {
    1
}

fn is_one(start: &u64) -> bool {
    *start == 1
}

impl BlockKind {
    pub fn heading(level: u8) -> Self {
        Self::Heading {
            level: level.clamp(1, 6),
        }
    }

    pub fn code(language: impl Into<String>) -> Self {
        Self::CodeBlock {
            language: language.into(),
        }
    }

    pub fn bullet_list() -> Self {
        Self::List {
            ordered: false,
            start: 1,
        }
    }

    pub fn ordered_list(start: u64) -> Self {
        Self::List {
            ordered: true,
            start,
        }
    }

    pub fn content(&self) -> ContentKind {
        match self {
            Self::Paragraph | Self::Heading { .. } | Self::CodeBlock { .. } => ContentKind::Inline,
            Self::Quote | Self::List { .. } | Self::ListItem => ContentKind::Blocks,
            Self::Rule => ContentKind::Atom,
            Self::Custom { content, .. } => match content {
                CustomContent::Inline => ContentKind::Inline,
                CustomContent::Blocks => ContentKind::Blocks,
                CustomContent::Atom => ContentKind::Atom,
            },
        }
    }

    /// Code holds plain text only: no marks, no inline nodes.
    pub fn is_code(&self) -> bool {
        matches!(self, Self::CodeBlock { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Inline {
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Marks::is_empty")]
        marks: Marks,
    },
    HardBreak,
    /// A caller's inline node (mention, emoji, ...). Counts as one character.
    Node {
        name: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        attrs: Attrs,
    },
}

impl Inline {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            marks: Marks::new(),
        }
    }

    pub fn marked(text: impl Into<String>, marks: Marks) -> Self {
        Self::Text {
            text: text.into(),
            marks,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Text { text, .. } => text.chars().count(),
            Self::HardBreak | Self::Node { .. } => 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn marks(&self) -> Option<&Marks> {
        match self {
            Self::Text { marks, .. } => Some(marks),
            _ => None,
        }
    }
}

/// A block's content, matching its kind's [`ContentKind`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Content {
    Inlines(Vec<Inline>),
    Blocks(Vec<Block>),
    Empty,
}

/// Keys are runtime identity, not stored: a loaded doc gets fresh ones.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "BlockRepr", into = "BlockRepr")]
pub struct Block {
    pub key: NodeKey,
    pub kind: BlockKind,
    pub content: Content,
}

#[derive(Clone, Serialize, Deserialize)]
struct BlockRepr {
    #[serde(flatten)]
    kind: BlockKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    inlines: Vec<Inline>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<Block>,
}

impl From<BlockRepr> for Block {
    fn from(repr: BlockRepr) -> Self {
        let mut block = Block {
            key: NodeKey(0),
            kind: repr.kind,
            content: Content::Inlines(repr.inlines),
        };
        if !repr.children.is_empty() {
            block.content = Content::Blocks(repr.children);
        }
        block
    }
}

impl From<Block> for BlockRepr {
    fn from(block: Block) -> Self {
        let (inlines, children) = match block.content {
            Content::Inlines(inlines) => (inlines, Vec::new()),
            Content::Blocks(children) => (Vec::new(), children),
            Content::Empty => (Vec::new(), Vec::new()),
        };
        Self {
            kind: block.kind,
            inlines,
            children,
        }
    }
}

impl Block {
    pub fn new(key: NodeKey, kind: BlockKind) -> Self {
        let content = match kind.content() {
            ContentKind::Inline => Content::Inlines(Vec::new()),
            ContentKind::Blocks => Content::Blocks(Vec::new()),
            ContentKind::Atom => Content::Empty,
        };
        Self { key, kind, content }
    }

    /// The inline content; empty for other blocks.
    pub fn inlines(&self) -> &[Inline] {
        match &self.content {
            Content::Inlines(inlines) => inlines,
            _ => &[],
        }
    }

    /// The inline content, made inline first if it was not.
    pub fn inlines_mut(&mut self) -> &mut Vec<Inline> {
        if !matches!(self.content, Content::Inlines(_)) {
            self.content = Content::Inlines(Vec::new());
        }
        match &mut self.content {
            Content::Inlines(inlines) => inlines,
            _ => unreachable!("just made inline"),
        }
    }

    pub fn children(&self) -> &[Block] {
        match &self.content {
            Content::Blocks(children) => children,
            _ => &[],
        }
    }

    /// The child blocks, made a container first if it was not.
    pub fn children_mut(&mut self) -> &mut Vec<Block> {
        if !matches!(self.content, Content::Blocks(_)) {
            self.content = Content::Blocks(Vec::new());
        }
        match &mut self.content {
            Content::Blocks(children) => children,
            _ => unreachable!("just made a container"),
        }
    }

    pub fn into_children(self) -> Vec<Block> {
        match self.content {
            Content::Blocks(children) => children,
            _ => Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        match self.kind.content() {
            ContentKind::Inline => inline_len(self.inlines()),
            ContentKind::Atom => 1,
            ContentKind::Blocks => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_leaf(&self) -> bool {
        self.kind.content() != ContentKind::Blocks
    }

    /// The plain text of an inline block; hard breaks read as `\n`.
    pub fn text(&self) -> String {
        self.inlines()
            .iter()
            .map(|inline| match inline {
                Inline::Text { text, .. } => text.as_str(),
                Inline::HardBreak => "\n",
                Inline::Node { .. } => "\u{fffc}",
            })
            .collect()
    }
}

/// The format of record. `version` lets a later model migrate older JSON.
///
/// Equality compares content only: two docs built apart never share keys.
#[derive(Clone, Debug, Serialize)]
pub struct Doc {
    pub version: u32,
    pub blocks: Vec<Block>,
    #[serde(skip)]
    next_key: u64,
}

impl PartialEq for Doc {
    fn eq(&self, other: &Self) -> bool {
        fn same(a: &[Block], b: &[Block]) -> bool {
            a.len() == b.len()
                && a.iter().zip(b).all(|(a, b)| {
                    a.kind == b.kind
                        && match (&a.content, &b.content) {
                            (Content::Blocks(x), Content::Blocks(y)) => same(x, y),
                            (x, y) => x == y,
                        }
                })
        }
        self.version == other.version && same(&self.blocks, &other.blocks)
    }
}

impl Eq for Doc {}

#[derive(Deserialize)]
struct DocRepr {
    #[serde(default = "current_version")]
    version: u32,
    #[serde(default)]
    blocks: Vec<Block>,
}

fn current_version() -> u32 {
    Doc::VERSION
}

impl<'de> Deserialize<'de> for Doc {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let repr = DocRepr::deserialize(deserializer)?;
        let mut doc = Doc {
            version: repr.version,
            blocks: repr.blocks,
            next_key: 1,
        };
        doc.assign_keys();
        doc.normalize();
        Ok(doc)
    }
}

impl Default for Doc {
    fn default() -> Self {
        Self::new()
    }
}

impl Doc {
    pub const VERSION: u32 = 1;

    /// One empty paragraph: a doc always has a place for the caret.
    pub fn new() -> Self {
        let mut doc = Self::empty();
        let paragraph = doc.leaf(BlockKind::Paragraph, Vec::new());
        doc.blocks.push(paragraph);
        doc
    }

    pub(crate) fn empty() -> Self {
        Self {
            version: Self::VERSION,
            blocks: Vec::new(),
            next_key: 1,
        }
    }

    pub fn key(&mut self) -> NodeKey {
        let key = NodeKey(self.next_key);
        self.next_key += 1;
        key
    }

    pub fn leaf(&mut self, kind: BlockKind, inlines: Vec<Inline>) -> Block {
        Block {
            key: self.key(),
            kind,
            content: Content::Inlines(inlines),
        }
    }

    pub fn container(&mut self, kind: BlockKind, children: Vec<Block>) -> Block {
        Block {
            key: self.key(),
            kind,
            content: Content::Blocks(children),
        }
    }

    /// Gives every block a fresh key, as after loading.
    fn assign_keys(&mut self) {
        fn assign(blocks: &mut [Block], next: &mut u64) {
            for block in blocks {
                block.key = NodeKey(*next);
                *next += 1;
                if let Content::Blocks(children) = &mut block.content {
                    assign(children, next);
                }
            }
        }
        let mut next = 1;
        assign(&mut self.blocks, &mut next);
        self.next_key = next;
    }

    /// Fixes what a hand-written or stored doc may get wrong: duplicate keys,
    /// split text runs, content in the wrong field, no block at all.
    pub fn normalize(&mut self) {
        let mut seen = std::collections::HashSet::new();
        let mut max = 0;
        visit(&self.blocks, &mut |block| max = max.max(block.key.0));
        self.next_key = self.next_key.max(max + 1);
        let mut blocks = std::mem::take(&mut self.blocks);
        for block in &mut blocks {
            self.normalize_block(block, &mut seen);
        }
        self.blocks = blocks;
        if self.blocks.is_empty() {
            let paragraph = self.leaf(BlockKind::Paragraph, Vec::new());
            self.blocks.push(paragraph);
        }
    }

    fn normalize_block(
        &mut self,
        block: &mut Block,
        seen: &mut std::collections::HashSet<NodeKey>,
    ) {
        if !seen.insert(block.key) {
            block.key = self.key();
            seen.insert(block.key);
        }
        if let BlockKind::Heading { level } = &mut block.kind {
            *level = (*level).clamp(1, 6);
        }
        match block.kind.content() {
            ContentKind::Inline => {
                if block.kind.is_code() {
                    let text = block.text();
                    *block.inlines_mut() = vec![Inline::text(text)];
                }
                normalize_inlines(block.inlines_mut());
            }
            ContentKind::Atom => block.content = Content::Empty,
            ContentKind::Blocks => {
                if matches!(block.kind, BlockKind::List { .. }) {
                    for child in block.children_mut() {
                        if child.kind != BlockKind::ListItem {
                            let inner = std::mem::replace(
                                child,
                                self.container(BlockKind::ListItem, Vec::new()),
                            );
                            child.children_mut().push(inner);
                        }
                    }
                }
                if block.children().is_empty() {
                    let paragraph = self.leaf(BlockKind::Paragraph, Vec::new());
                    block.children_mut().push(paragraph);
                }
                for child in block.children_mut() {
                    self.normalize_block(child, seen);
                }
            }
        }
    }

    /// The path of child indices to `key`, outermost first.
    pub fn path(&self, key: NodeKey) -> Option<Vec<usize>> {
        fn find(blocks: &[Block], key: NodeKey, path: &mut Vec<usize>) -> bool {
            for (index, block) in blocks.iter().enumerate() {
                path.push(index);
                if block.key == key || find(block.children(), key, path) {
                    return true;
                }
                path.pop();
            }
            false
        }
        let mut path = Vec::new();
        find(&self.blocks, key, &mut path).then_some(path)
    }

    pub fn at(&self, path: &[usize]) -> &Block {
        let (first, rest) = path.split_first().expect("a non-empty path");
        rest.iter().fold(&self.blocks[*first], |block, index| {
            &block.children()[*index]
        })
    }

    pub fn at_mut(&mut self, path: &[usize]) -> &mut Block {
        let (first, rest) = path.split_first().expect("a non-empty path");
        rest.iter().fold(&mut self.blocks[*first], |block, index| {
            &mut block.children_mut()[*index]
        })
    }

    /// The sibling list holding the block at `path`.
    pub fn siblings_mut(&mut self, path: &[usize]) -> &mut Vec<Block> {
        match path.split_last() {
            Some((_, [])) | None => &mut self.blocks,
            Some((_, parent)) => self.at_mut(parent).children_mut(),
        }
    }

    pub fn get(&self, key: NodeKey) -> Option<&Block> {
        self.path(key).map(|path| self.at(&path))
    }

    pub fn get_mut(&mut self, key: NodeKey) -> Option<&mut Block> {
        self.path(key).map(|path| self.at_mut(&path))
    }

    /// Inline and atom blocks in reading order: where a caret can be.
    pub fn leaves(&self) -> Vec<NodeKey> {
        let mut keys = Vec::new();
        visit(&self.blocks, &mut |block| {
            if block.is_leaf() {
                keys.push(block.key);
            }
        });
        keys
    }

    pub fn first_leaf(&self) -> NodeKey {
        self.leaves()[0]
    }

    pub fn last_leaf(&self) -> NodeKey {
        *self.leaves().last().expect("a doc has a block")
    }

    /// The doc with keys renumbered in reading order: compares content, not identity.
    pub fn rekeyed(&self) -> Self {
        fn renumber(blocks: &mut [Block], next: &mut u64) {
            for block in blocks {
                block.key = NodeKey(*next);
                *next += 1;
                if let Content::Blocks(children) = &mut block.content {
                    renumber(children, next);
                }
            }
        }
        let mut doc = self.clone();
        let mut next = 1;
        renumber(&mut doc.blocks, &mut next);
        doc.next_key = next;
        doc
    }

    /// The plain text, one line per leaf.
    pub fn plain_text(&self) -> String {
        let mut lines = Vec::new();
        visit(&self.blocks, &mut |block| {
            if block.kind.content() == ContentKind::Inline {
                lines.push(block.text());
            }
        });
        lines.join("\n")
    }
}

/// Visits every block depth first, parents before children.
pub fn visit<'a>(blocks: &'a [Block], f: &mut impl FnMut(&'a Block)) {
    for block in blocks {
        f(block);
        visit(block.children(), f);
    }
}

pub fn inline_len(inlines: &[Inline]) -> usize {
    inlines.iter().map(Inline::len).sum()
}

/// Merges neighbouring text runs with equal marks and drops empty ones.
pub fn normalize_inlines(inlines: &mut Vec<Inline>) {
    let mut merged: Vec<Inline> = Vec::with_capacity(inlines.len());
    for inline in inlines.drain(..) {
        if let Inline::Text { text, marks } = &inline {
            if text.is_empty() {
                continue;
            }
            if let Some(Inline::Text {
                text: last,
                marks: last_marks,
            }) = merged.last_mut()
                && last_marks == marks
            {
                last.push_str(text);
                continue;
            }
        }
        merged.push(inline);
    }
    *inlines = merged;
}

/// Splits at a char offset; a run straddling it is cut in two.
pub fn split_inlines(inlines: &[Inline], offset: usize) -> (Vec<Inline>, Vec<Inline>) {
    let mut before = Vec::new();
    let mut after = Vec::new();
    let mut at = 0;
    for inline in inlines {
        let len = inline.len();
        if at + len <= offset {
            before.push(inline.clone());
        } else if at >= offset {
            after.push(inline.clone());
        } else if let Inline::Text { text, marks } = inline {
            let cut = byte_index(text, offset - at);
            before.push(Inline::marked(&text[..cut], marks.clone()));
            after.push(Inline::marked(&text[cut..], marks.clone()));
        }
        at += len;
    }
    (before, after)
}

/// The inlines between two char offsets.
pub fn slice_inlines(inlines: &[Inline], from: usize, to: usize) -> Vec<Inline> {
    let (_, rest) = split_inlines(inlines, from);
    split_inlines(&rest, to.saturating_sub(from)).0
}

pub fn byte_index(text: &str, chars: usize) -> usize {
    text.char_indices()
        .nth(chars)
        .map_or(text.len(), |(index, _)| index)
}

/// The marks typing at `offset` takes: the char before's, minus non-inclusive
/// marks that end there. At the start, the first char's inclusive marks.
pub fn marks_at(inlines: &[Inline], offset: usize) -> Marks {
    let mut at = 0;
    let mut before: Option<&Marks> = None;
    let mut after: Option<&Marks> = None;
    for inline in inlines {
        let len = inline.len();
        if at < offset && offset <= at + len {
            before = inline.marks();
        }
        if at <= offset && offset < at + len && after.is_none() {
            after = inline.marks();
        }
        at += len;
    }
    match (before, after) {
        (Some(before), after) => before
            .iter()
            .filter(|mark| {
                mark.inclusive() || after.is_some_and(|after| after.iter().any(|m| m == *mark))
            })
            .cloned()
            .collect(),
        (None, Some(after)) if offset == 0 => after
            .iter()
            .filter(|mark| mark.inclusive())
            .cloned()
            .collect(),
        _ => Marks::new(),
    }
}

/// Runs `change` over the marks of every text run between the offsets.
pub fn map_marks(inlines: &mut Vec<Inline>, from: usize, to: usize, change: impl Fn(&mut Marks)) {
    let (before, rest) = split_inlines(inlines, from);
    let (mut middle, after) = split_inlines(&rest, to - from);
    for inline in &mut middle {
        if let Inline::Text { marks, .. } = inline {
            change(marks);
        }
    }
    *inlines = before.into_iter().chain(middle).chain(after).collect();
    normalize_inlines(inlines);
}

/// Whether every text char between the offsets carries a mark of `kind`; `false` with no text.
pub fn all_marked(inlines: &[Inline], from: usize, to: usize, kind: MarkKind) -> bool {
    let slice = slice_inlines(inlines, from, to);
    let mut any = false;
    for inline in &slice {
        if let Inline::Text { marks, .. } = inline {
            if !marks.has(kind) {
                return false;
            }
            any = true;
        }
    }
    any
}

/// The char range of the run carrying exactly this `mark` around `offset`.
pub fn mark_range(inlines: &[Inline], offset: usize, mark: &Mark) -> Option<(usize, usize)> {
    let mut at = 0;
    let mut range: Option<(usize, usize)> = None;
    for inline in inlines {
        let len = inline.len();
        let carries = inline
            .marks()
            .is_some_and(|marks| marks.iter().any(|m| m == mark));
        match (carries, range) {
            (true, Some((start, end))) if end == at => range = Some((start, at + len)),
            (true, _) if range.is_none_or(|(start, end)| !(start <= offset && offset <= end)) => {
                range = Some((at, at + len))
            }
            _ => {}
        }
        at += len;
    }
    range.filter(|(start, end)| *start <= offset && offset <= *end)
}
