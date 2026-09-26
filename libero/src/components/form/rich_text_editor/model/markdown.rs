//! The Markdown emitter; [`super::parse`] reads it back.

use super::doc::{Block, BlockKind, ContentKind, Doc, Inline, normalize_inlines, visit};
use super::mark::{Mark, MarkKind, Marks};
use super::registry::NodeRegistry;
use super::syntax::valid_inner;

impl Doc {
    pub fn to_markdown(&self) -> String {
        self.to_markdown_with(&NodeRegistry::default())
    }

    /// Writes caller nodes through their registered `to_markdown`.
    pub fn to_markdown_with(&self, registry: &NodeRegistry) -> String {
        let mut out = Writer { registry }.blocks(&self.blocks);
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }

    /// One line per text leaf, as [`Doc::plain_text`]; caller inline nodes write
    /// through their registered `to_markdown` instead of a placeholder.
    pub(crate) fn plain_text_with(&self, registry: &NodeRegistry) -> String {
        let mut lines = Vec::new();
        visit(&self.blocks, &mut |block| {
            if block.kind.content() != ContentKind::Inline {
                return;
            }
            let line: String = block
                .inlines()
                .iter()
                .map(|inline| match inline {
                    Inline::Text { text, .. } => text.clone(),
                    Inline::HardBreak => "\n".to_string(),
                    Inline::Node { name, attrs } => registry
                        .get(name)
                        .and_then(|spec| spec.to_markdown.as_ref())
                        .map(|write| write(attrs, ""))
                        .unwrap_or_default(),
                })
                .collect();
            lines.push(line);
        });
        lines.join("\n")
    }
}

struct Writer<'a> {
    registry: &'a NodeRegistry,
}

impl Writer<'_> {
    fn blocks(&self, blocks: &[Block]) -> String {
        let mut parts: Vec<String> = Vec::new();
        let mut previous_list: Option<(bool, bool)> = None;
        for block in blocks {
            let alternate = match (&block.kind, previous_list) {
                // Two lists in a row merge unless their markers differ.
                (BlockKind::List { ordered, .. }, Some((last_ordered, last_alternate))) => {
                    *ordered == last_ordered && !last_alternate
                }
                _ => false,
            };
            let text = self.block(block, alternate);
            if text.is_empty() {
                continue;
            }
            previous_list = match block.kind {
                BlockKind::List { ordered, .. } => Some((ordered, alternate)),
                _ => None,
            };
            parts.push(text);
        }
        parts.join("\n\n")
    }

    fn block(&self, block: &Block, alternate: bool) -> String {
        match &block.kind {
            BlockKind::Paragraph => escape_line_starts(&self.inlines(block.inlines())),
            BlockKind::Heading { level } => {
                let text = self.inlines(&flatten_breaks(block.inlines()));
                format!(
                    "{} {}",
                    "#".repeat(*level as usize),
                    escape_line_starts(&text)
                )
                .trim_end()
                .to_string()
            }
            BlockKind::CodeBlock { language } => code_block(&block.text(), language),
            BlockKind::Quote => prefix_lines(&self.blocks(block.children()), "> ", ">"),
            BlockKind::List { ordered, start } => self.list(block, *ordered, *start, alternate),
            BlockKind::ListItem => self.blocks(block.children()),
            BlockKind::Rule => "---".to_string(),
            BlockKind::Custom { name, attrs, .. } => {
                let content = match block.kind.content() {
                    ContentKind::Inline => self.inlines(block.inlines()),
                    ContentKind::Blocks => self.blocks(block.children()),
                    ContentKind::Atom => String::new(),
                };
                match self
                    .registry
                    .get(name)
                    .and_then(|spec| spec.to_markdown.as_ref())
                {
                    Some(write) => write(attrs, &content),
                    None => content,
                }
            }
        }
    }

    fn list(&self, block: &Block, ordered: bool, start: u64, alternate: bool) -> String {
        let loose = block
            .children()
            .iter()
            .any(|item| self.item(item.children()).contains("\n\n"));
        let items: Vec<String> = block
            .children()
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let marker = match (ordered, alternate) {
                    (false, false) => "-".to_string(),
                    (false, true) => "*".to_string(),
                    (true, false) => format!("{}.", start + index as u64),
                    (true, true) => format!("{})", start + index as u64),
                };
                let content = self.item(item.children());
                let indent = " ".repeat(marker.len() + 1);
                match content.is_empty() {
                    true => marker,
                    false => {
                        let body = prefix_lines(&content, &indent, "");
                        format!("{marker} {}", &body[indent.len()..])
                    }
                }
            })
            .collect();
        items.join(if loose { "\n\n" } else { "\n" })
    }

    /// An item's blocks: a list right after a paragraph needs no blank line, which keeps the list tight.
    fn item(&self, children: &[Block]) -> String {
        let mut out = String::new();
        for (index, child) in children.iter().enumerate() {
            let text = self.block(child, false);
            if text.is_empty() {
                continue;
            }
            if !out.is_empty() {
                let interrupts = matches!(child.kind, BlockKind::List { .. });
                let after_paragraph = index > 0 && children[index - 1].kind == BlockKind::Paragraph;
                out.push_str(if interrupts && after_paragraph {
                    "\n"
                } else {
                    "\n\n"
                });
            }
            out.push_str(&text);
        }
        out
    }

    /// Delimiters read best; where the subset would misread them (`***a***`, a span
    /// across a break), the paragraph falls back to HTML tags.
    fn inlines(&self, inlines: &[Inline]) -> String {
        let write = |tags: bool| {
            InlineWriter {
                tags,
                ..InlineWriter::default()
            }
            .write(inlines, self.registry)
        };
        let delimited = write(false);
        let has_nodes = inlines
            .iter()
            .any(|inline| matches!(inline, Inline::Node { .. }));
        if has_nodes || reads_back(&delimited, inlines) {
            return delimited;
        }
        let tagged = write(true);
        match reads_back(&tagged, inlines) {
            true => tagged,
            false => delimited,
        }
    }
}

/// Opens and closes mark delimiters around runs, keeping them properly nested.
#[derive(Default)]
struct InlineWriter {
    out: String,
    open: Vec<Mark>,
    code: Option<String>,
    pending_space: String,
    /// HTML tags instead of `*`, `**`, `~~`: no flanking rules to trip over.
    tags: bool,
}

impl InlineWriter {
    fn write(mut self, inlines: &[Inline], registry: &NodeRegistry) -> String {
        let marks_of = |inline: &Inline| match inline {
            Inline::Text { marks, .. } => Some(marks.clone()),
            _ => None,
        };
        for (index, inline) in inlines.iter().enumerate() {
            let target = match marks_of(inline) {
                Some(marks) => marks,
                None => {
                    // A break or node keeps the marks both neighbours share, never code.
                    let before = inlines[..index]
                        .iter()
                        .rev()
                        .find_map(marks_of)
                        .unwrap_or_default();
                    let after = inlines[index + 1..]
                        .iter()
                        .find_map(marks_of)
                        .unwrap_or_default();
                    before
                        .iter()
                        .filter(|mark| {
                            mark.kind() != MarkKind::Code && after.iter().any(|m| m == *mark)
                        })
                        .cloned()
                        .collect()
                }
            };
            let text = match inline {
                Inline::Text { text, .. } => text.clone(),
                _ => String::new(),
            };
            self.close_to(&target);
            self.flush_space();
            let opening = target.iter().any(|mark| !self.open.contains(mark));
            // A delimiter followed by space would not open: the space goes before it.
            let body = match opening && !target.has(MarkKind::Code) {
                true => {
                    let (lead, body) = split_leading_space(&text);
                    self.out.push_str(lead);
                    body
                }
                false => text.as_str(),
            };
            self.open_marks(&target, inlines, index);
            match inline {
                Inline::Text { .. } => self.push_text(body),
                Inline::HardBreak => self.out.push_str("\\\n"),
                Inline::Node { name, attrs } => {
                    let written = registry
                        .get(name)
                        .and_then(|spec| spec.to_markdown.as_ref())
                        .map(|write| write(attrs, ""))
                        .unwrap_or_default();
                    self.out.push_str(&written);
                }
            }
        }
        self.close_to(&Marks::new());
        self.flush_space();
        self.out
    }

    fn push_text(&mut self, text: &str) {
        if let Some(code) = &mut self.code {
            code.push_str(text);
            return;
        }
        let trimmed = text.trim_end_matches([' ', '\t']);
        self.out.push_str(&escape(trimmed));
        self.pending_space = text[trimmed.len()..].to_string();
    }

    fn flush_space(&mut self) {
        let space = std::mem::take(&mut self.pending_space);
        self.out.push_str(&space);
    }

    /// Closes open marks until every one left is in `target`.
    /// An open code span closes too when an outer mark opens: code stays innermost.
    fn close_to(&mut self, target: &Marks) {
        let adds_outer = target
            .iter()
            .any(|mark| mark.kind() != MarkKind::Code && !self.open.contains(mark));
        let code = self
            .open
            .iter()
            .position(|mark| *mark == Mark::Code)
            .filter(|_| adds_outer);
        let stale = self
            .open
            .iter()
            .position(|mark| !target.iter().any(|m| m == mark));
        let Some(first_stale) = [stale, code].into_iter().flatten().min() else {
            return;
        };
        while self.open.len() > first_stale {
            let mark = self.open.pop().expect("an open mark");
            if mark == Mark::Code {
                let code = self.code.take().unwrap_or_default();
                self.out.push_str(&code_span(&code));
                continue;
            }
            // Closing after trailing space would not close: the space goes after.
            let space = std::mem::take(&mut self.pending_space);
            self.out.push_str(&closer(&mark, self.tags));
            self.pending_space = space;
        }
    }

    /// Opens what `target` adds, the longest-running first and code last.
    fn open_marks(&mut self, target: &Marks, inlines: &[Inline], index: usize) {
        let extent = |mark: &Mark| {
            inlines[index..]
                .iter()
                .take_while(|inline| match inline {
                    Inline::Text { marks, .. } => marks.iter().any(|m| m == mark),
                    _ => mark.kind() != MarkKind::Code,
                })
                .count()
        };
        let mut new: Vec<Mark> = target
            .iter()
            .filter(|mark| !self.open.contains(mark))
            .cloned()
            .collect();
        new.sort_by_key(|mark| {
            (
                mark.kind() == MarkKind::Code,
                std::cmp::Reverse(extent(mark)),
            )
        });
        for mark in new {
            match mark {
                Mark::Code => self.code = Some(String::new()),
                _ => self.out.push_str(opener(&mark, self.tags)),
            }
            self.open.push(mark);
        }
    }
}

fn reads_back(written: &str, inlines: &[Inline]) -> bool {
    let doc = Doc::from_markdown(&escape_line_starts(written));
    let mut expected = inlines.to_vec();
    normalize_inlines(&mut expected);
    match doc.blocks.as_slice() {
        [block] => block.inlines() == expected,
        _ => expected.is_empty(),
    }
}

fn split_leading_space(text: &str) -> (&str, &str) {
    let body = text.trim_start_matches([' ', '\t']);
    (&text[..text.len() - body.len()], body)
}

fn opener(mark: &Mark, tags: bool) -> &'static str {
    match mark {
        Mark::Bold if tags => "<strong>",
        Mark::Italic if tags => "<em>",
        Mark::Strike if tags => "<s>",
        Mark::Bold => "**",
        Mark::Italic => "*",
        Mark::Underline => "<u>",
        Mark::Strike => "~~",
        Mark::Code => "`",
        Mark::Link { .. } => "[",
    }
}

fn closer(mark: &Mark, tags: bool) -> String {
    match mark {
        Mark::Underline | Mark::Bold | Mark::Italic | Mark::Strike
            if tags || *mark == Mark::Underline =>
        {
            opener(mark, true).replacen('<', "</", 1)
        }
        Mark::Link { href, title } => {
            let title = title
                .as_ref()
                .map(|title| format!(" \"{}\"", title.replace('\\', "\\\\").replace('"', "\\\"")))
                .unwrap_or_default();
            format!("]({}{title})", destination(href.as_str()))
        }
        other => opener(other, false).to_string(),
    }
}

fn destination(href: &str) -> String {
    let escaped: String = href
        .chars()
        .flat_map(|c| match c {
            '\\' | '(' | ')' | '<' | '>' => vec!['\\', c],
            _ => vec![c],
        })
        .collect();
    match href.contains(' ') {
        true => format!("<{escaped}>"),
        false => escaped,
    }
}

/// `` `code` `` as typed; code the subset cannot delimit (a backtick, edge spaces) uses `<code>`.
fn code_span(code: &str) -> String {
    let chars: Vec<char> = code.chars().collect();
    match valid_inner(&chars, "`") {
        true => format!("`{code}`"),
        false => format!("<code>{}</code>", escape(code)),
    }
}

/// A language with a backtick cannot follow a backtick fence: those backticks are dropped.
fn code_block(code: &str, language: &str) -> String {
    let language = language.replace('`', "");
    let fence = "`".repeat((longest_run(code, '`') + 1).max(3));
    match code.is_empty() {
        true => format!("{fence}{language}\n{fence}"),
        false => format!("{fence}{language}\n{code}\n{fence}"),
    }
}

fn longest_run(text: &str, c: char) -> usize {
    let mut longest = 0;
    let mut run = 0;
    for ch in text.chars() {
        run = if ch == c { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    longest
}

/// Backslash-escapes every char that could start Markdown syntax inside a line.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(
            c,
            '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '~' | '!' | '#'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Escapes what only means something at a line start: list markers.
fn escape_line_starts(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            let body = line.trim_start_matches(' ');
            let lead = &line[..line.len() - body.len()];
            let digits = body.chars().take_while(char::is_ascii_digit).count();
            match body.chars().next() {
                Some('-' | '+') => format!("{lead}\\{body}"),
                Some(_)
                    if digits > 0 && matches!(body[digits..].chars().next(), Some('.' | ')')) =>
                {
                    format!("{lead}{}\\{}", &body[..digits], &body[digits..])
                }
                _ => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn prefix_lines(text: &str, prefix: &str, empty: &str) -> String {
    text.split('\n')
        .map(|line| match line.is_empty() {
            true => empty.to_string(),
            false => format!("{prefix}{line}"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A heading is one line: its hard breaks become spaces.
fn flatten_breaks(inlines: &[Inline]) -> Vec<Inline> {
    inlines
        .iter()
        .map(|inline| match inline {
            Inline::HardBreak => Inline::text(" "),
            other => other.clone(),
        })
        .collect()
}
