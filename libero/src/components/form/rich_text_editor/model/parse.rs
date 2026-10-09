//! The Markdown reader. It reads the editor's own subset, not CommonMark, through the
//! recognizers the typing shortcuts use ([`super::syntax`]); anything else stays text.
//!
//! - Blocks: `#` headings, `-`/`*`/`+` and `1.`/`1)` lists (nested by indent), `>` quotes,
//!   `---`/`***`/`___` rules, backtick fences. Any of them ends a paragraph.
//! - Inline: `` `code` ``, `**bold**`, `*italic*`, `~~strike~~` as typed, `[text](url "title")`,
//!   `<u>`, `<strong>`, `<em>`, `<s>`, `<code>` tags, backslash escapes and `\`-newline breaks.

use super::doc::{Block, BlockKind, Doc, Inline};
use super::mark::{Mark, Marks};
use super::syntax::{BlockSyntax, DELIMITERS, block_syntax, closing_span, link, valid_inner};

impl Doc {
    pub fn from_markdown(markdown: &str) -> Self {
        Self::read(markdown, false)
    }

    /// Pasted text from elsewhere: Markdown, but each line its own paragraph, as
    /// plain text means it. A `\`-newline break still joins.
    pub(crate) fn from_pasted(text: &str) -> Self {
        Self::read(text, true)
    }

    fn read(markdown: &str, per_line: bool) -> Self {
        let lines: Vec<&str> = markdown.lines().collect();
        let mut doc = Doc::empty();
        doc.blocks = blocks(&mut doc, &lines, per_line);
        doc.normalize();
        doc
    }
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// The block a line starts and the text after its prefix.
fn line_syntax(line: &str) -> Option<(BlockSyntax, &str)> {
    let line = line.trim_start();
    let whole = |syntax: &BlockSyntax| matches!(syntax, BlockSyntax::Fence(..) | BlockSyntax::Rule);
    if let Some(syntax) = block_syntax(line.trim_end()).filter(whole) {
        return Some((syntax, ""));
    }
    let (word, rest) = line.split_once(' ').unwrap_or((line, ""));
    block_syntax(word)
        .filter(|syntax| !whole(syntax))
        .map(|syntax| (syntax, rest))
}

fn blocks(doc: &mut Doc, lines: &[&str], per_line: bool) -> Vec<Block> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        i += 1;
        let Some((syntax, rest)) = line_syntax(line) else {
            if blank(line) {
                continue;
            }
            let mut text = vec![line.trim()];
            while let Some(next) = lines.get(i).filter(|next| {
                !blank(next)
                    && line_syntax(next).is_none()
                    && (!per_line || text.last().is_some_and(|last| last.ends_with('\\')))
            }) {
                text.push(next.trim());
                i += 1;
            }
            out.push(doc.leaf(BlockKind::Paragraph, inlines(&text.join("\n"))));
            continue;
        };
        let block = match syntax {
            BlockSyntax::Heading(level) => {
                doc.leaf(BlockKind::heading(level), inlines(rest.trim()))
            }
            BlockSyntax::Rule => doc.leaf(BlockKind::Rule, Vec::new()),
            BlockSyntax::Fence(len, info) => {
                let mut code = Vec::new();
                while let Some(next) = lines.get(i) {
                    i += 1;
                    let trimmed = next.trim();
                    if trimmed.len() >= len && trimmed.chars().all(|c| c == '`') {
                        break;
                    }
                    code.push(&next[indent(next).min(indent(line))..]);
                }
                doc.leaf(BlockKind::code(info), vec![Inline::text(code.join("\n"))])
            }
            BlockSyntax::Quote => {
                let mut inner = vec![rest];
                while let Some((BlockSyntax::Quote, rest)) =
                    lines.get(i).and_then(|l| line_syntax(l))
                {
                    inner.push(rest);
                    i += 1;
                }
                let children = blocks(doc, &inner, per_line);
                doc.container(BlockKind::Quote, children)
            }
            BlockSyntax::Bullet(_) | BlockSyntax::Ordered(..) => {
                let same = |other: &BlockSyntax| match (&syntax, other) {
                    (BlockSyntax::Bullet(a), BlockSyntax::Bullet(b)) => a == b,
                    (BlockSyntax::Ordered(_, a), BlockSyntax::Ordered(_, b)) => a == b,
                    _ => false,
                };
                let mut items = Vec::new();
                let mut head = (line, rest);
                loop {
                    // Continuation lines are indented past the marker.
                    let width = indent(head.0) + head.0.trim_start().len() - head.1.len();
                    let mut inner = vec![head.1];
                    while let Some(next) = lines
                        .get(i)
                        .filter(|next| blank(next) || indent(next) >= width)
                    {
                        inner.push(next.get(width..).unwrap_or(""));
                        i += 1;
                    }
                    let children = blocks(doc, &inner, per_line);
                    items.push(doc.container(BlockKind::ListItem, children));
                    match lines.get(i).and_then(|l| Some((*l, line_syntax(l)?))) {
                        Some((next, (other, rest))) if same(&other) => {
                            head = (next, rest);
                            i += 1;
                        }
                        _ => break,
                    }
                }
                let kind = match syntax {
                    BlockSyntax::Ordered(start, _) => BlockKind::ordered_list(start),
                    _ => BlockKind::bullet_list(),
                };
                doc.container(kind, items)
            }
        };
        out.push(block);
    }
    out
}

struct Cell {
    c: char,
    marks: Marks,
    /// Whether the char may act as a delimiter: not escaped, not inside code.
    active: bool,
}

/// The only HTML read as marks; any other tag stays literal text.
const TAG_MARKS: [(&str, Mark); 5] = [
    ("u", Mark::Underline),
    ("strong", Mark::Bold),
    ("em", Mark::Italic),
    ("s", Mark::Strike),
    ("code", Mark::Code),
];

/// Lexes escapes, code spans, tags and links, then replays the text as if typed.
fn inlines(text: &str) -> Vec<Inline> {
    let chars: Vec<char> = text.chars().collect();
    let mut cells: Vec<Cell> = Vec::new();
    let mut tags = [0usize; TAG_MARKS.len()];
    // Open links: the index of their `]`, the index after their `)`, the mark.
    let mut links: Vec<(usize, usize, Mark)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if let Some((_, end, _)) = links.last().filter(|link| link.0 <= i) {
            i = *end;
            links.pop();
            continue;
        }
        let marks: Marks = links
            .iter()
            .map(|link| link.2.clone())
            .chain(
                TAG_MARKS
                    .iter()
                    .zip(tags)
                    .filter(|(_, n)| *n > 0)
                    .map(|((_, m), _)| m.clone()),
            )
            .collect();
        let next = chars.get(i + 1).copied();
        let (c, active, len) = match chars[i] {
            '\\' if next == Some('\n') => ('\n', false, 2),
            '\\' if next.is_some_and(|c| c.is_ascii_punctuation()) => (chars[i + 1], false, 2),
            '\n' => (' ', false, 1),
            '`' => match code_span(&chars, i) {
                Some(end) => {
                    for &c in &chars[i + 1..end] {
                        let marks = marks.clone().with(Mark::Code);
                        cells.push(Cell {
                            c,
                            marks,
                            active: false,
                        });
                    }
                    i = end + 1;
                    continue;
                }
                None => ('`', false, 1),
            },
            '<' => match tag(&chars, i) {
                Some((index, opening, end)) => {
                    tags[index] = if opening {
                        tags[index] + 1
                    } else {
                        tags[index].saturating_sub(1)
                    };
                    i = end;
                    continue;
                }
                None => ('<', false, 1),
            },
            // `![` is an image, outside the subset: it stays text.
            '[' if !cells
                .last()
                .is_some_and(|cell| cell.c == '!' && cell.active) =>
            {
                match link(&chars, i) {
                    Some(found) => {
                        links.push(found);
                        i += 1;
                        continue;
                    }
                    None => ('[', false, 1),
                }
            }
            c => (c, true, 1),
        };
        cells.push(Cell { c, marks, active });
        i += len;
    }
    let mut out: Vec<Inline> = Vec::new();
    for cell in replay(cells) {
        match (out.last_mut(), cell.c) {
            (_, '\n') => out.push(Inline::HardBreak),
            (Some(Inline::Text { text, marks, .. }), c) if *marks == cell.marks => text.push(c),
            (_, c) => out.push(Inline::marked(c.to_string(), cell.marks)),
        }
    }
    super::doc::normalize_inlines(&mut out);
    out
}

/// Feeds the chars through the typing shortcuts' span rule, one at a time.
fn replay(cells: Vec<Cell>) -> Vec<Cell> {
    let mut out: Vec<Cell> = Vec::with_capacity(cells.len());
    let mut text: Vec<char> = Vec::with_capacity(cells.len());
    for cell in cells {
        let live = cell.active && matches!(cell.c, '*' | '~');
        text.push(cell.c);
        out.push(cell);
        if !live {
            continue;
        }
        // Code spans were taken by the lexer.
        let Some((open, len, mark)) = closing_span(&text, |at| out[at].active, &DELIMITERS[1..])
        else {
            continue;
        };
        let close = out.len() - len;
        out.truncate(close);
        text.truncate(close);
        out.drain(open..open + len);
        text.drain(open..open + len);
        for cell in &mut out[open..] {
            cell.marks.add(mark.clone());
        }
    }
    out
}

/// The closing backtick of a code span opening at `at`.
fn code_span(chars: &[char], at: usize) -> Option<usize> {
    let end = at + 1 + chars[at + 1..].iter().position(|c| *c == '`')?;
    valid_inner(&chars[at + 1..end], "`").then_some(end)
}

/// A [`TAG_MARKS`] tag at `at`: its index, whether it opens, and the index after it.
fn tag(chars: &[char], at: usize) -> Option<(usize, bool, usize)> {
    let end = at + chars[at..].iter().take(10).position(|c| *c == '>')?;
    let inner: String = chars[at + 1..end].iter().collect();
    let (name, opening) = match inner.strip_prefix('/') {
        Some(name) => (name, false),
        None => (inner.as_str(), true),
    };
    let index = TAG_MARKS
        .iter()
        .position(|(tag, _)| name.eq_ignore_ascii_case(tag))?;
    Some((index, opening, end + 1))
}
