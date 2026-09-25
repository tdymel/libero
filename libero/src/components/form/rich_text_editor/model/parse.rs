//! A hand-written Markdown reader for the editor's subset: headings, paragraphs,
//! quotes, lists, rules, code blocks; emphasis, strike, code, links, `<u>`-style tags.
//! Follows CommonMark where the subset reaches; anything else stays literal text.

use super::doc::{Block, BlockKind, Doc, Inline};
use super::edit::coerce;
use super::mark::{Href, Mark, Marks};

impl Doc {
    pub fn from_markdown(markdown: &str) -> Self {
        let lines: Vec<String> = markdown.lines().map(expand_tabs).collect();
        let mut doc = Doc::empty();
        doc.blocks = blocks(&mut doc, &lines);
        doc.normalize();
        doc
    }
}

fn expand_tabs(line: &str) -> String {
    let body = line.trim_start_matches([' ', '\t']);
    let lead: String = line[..line.len() - body.len()]
        .chars()
        .map(|c| if c == '\t' { "    " } else { " " })
        .collect();
    lead + body
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// Up to three spaces of indent, as most block starts allow.
fn lead(line: &str) -> Option<&str> {
    (indent(line) < 4).then(|| line.trim_start_matches(' '))
}

/// An opening fence: its char, length, indent and info string.
fn fence(line: &str) -> Option<(char, usize, usize, String)> {
    let rest = lead(line)?;
    let ch = rest.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let len = rest.chars().take_while(|c| *c == ch).count();
    let info = rest[len..].trim();
    (len >= 3 && !(ch == '`' && info.contains('`')))
        .then(|| (ch, len, indent(line), info.to_string()))
}

fn closes_fence(line: &str, ch: char, len: usize) -> bool {
    lead(line).is_some_and(|rest| {
        let run = rest.chars().take_while(|c| *c == ch).count();
        run >= len && rest[run..].trim().is_empty()
    })
}

fn atx(line: &str) -> Option<(u8, String)> {
    let rest = lead(line)?;
    let level = rest.chars().take_while(|c| *c == '#').count();
    let text = &rest[level..];
    if !(1..=6).contains(&level) || !(text.is_empty() || text.starts_with(' ')) {
        return None;
    }
    let text = text.trim();
    let closed = text.trim_end_matches('#');
    let text = match closed.is_empty() || closed.ends_with(' ') {
        true => closed.trim_end(),
        false => text,
    };
    Some((level as u8, text.to_string()))
}

fn rule(line: &str) -> bool {
    let Some(rest) = lead(line) else {
        return false;
    };
    let Some(ch) = rest.chars().next().filter(|c| matches!(c, '-' | '*' | '_')) else {
        return false;
    };
    rest.chars().all(|c| c == ch || c == ' ') && rest.chars().filter(|c| *c == ch).count() >= 3
}

fn setext(line: &str) -> Option<u8> {
    let rest = lead(line)?.trim_end();
    let ch = rest.chars().next()?;
    let level = match ch {
        '=' => 1,
        '-' => 2,
        _ => return None,
    };
    rest.chars().all(|c| c == ch).then_some(level)
}

fn quote_rest(line: &str) -> Option<String> {
    let rest = lead(line)?.strip_prefix('>')?;
    Some(rest.strip_prefix(' ').unwrap_or(rest).to_string())
}

struct Marker {
    ordered: bool,
    /// The bullet char, or the delimiter after the number.
    ch: char,
    start: u64,
    width: usize,
    rest: String,
}

fn marker(line: &str) -> Option<Marker> {
    let rest = lead(line)?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    let (ordered, ch, start, len) = match rest.chars().next()? {
        c @ ('-' | '+' | '*') => (false, c, 1, 1),
        _ if (1..=9).contains(&digits) => {
            let delimiter = rest[digits..]
                .chars()
                .next()
                .filter(|c| matches!(c, '.' | ')'))?;
            (true, delimiter, rest[..digits].parse().ok()?, digits + 1)
        }
        _ => return None,
    };
    let after = &rest[len..];
    if !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    let spaces = indent(after);
    let before = indent(line) + len;
    let (width, text) = match spaces {
        _ if blank(after) => (before + 1, String::new()),
        1..=4 => (before + spaces, after[spaces..].to_string()),
        _ => (before + 1, after[1..].to_string()),
    };
    Some(Marker {
        ordered,
        ch,
        start,
        width,
        rest: text,
    })
}

/// Whether `line` starts a block that ends a paragraph.
fn interrupts(line: &str) -> bool {
    fence(line).is_some()
        || atx(line).is_some()
        || rule(line)
        || quote_rest(line).is_some()
        || marker(line).is_some_and(|m| !blank(&m.rest) && (!m.ordered || m.start == 1))
}

/// Whether the last line of `lines` sits in paragraph text, so a lazy line may continue it.
fn in_paragraph(lines: &[String]) -> bool {
    let fences = lines.iter().filter(|line| fence(line).is_some()).count();
    lines.last().is_some_and(|line| !blank(line)) && fences % 2 == 0
}

fn blocks(doc: &mut Doc, lines: &[String]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        if blank(line) {
            i += 1;
        } else if indent(line) >= 4 {
            let mut code = Vec::new();
            while i < lines.len() && (blank(&lines[i]) || indent(&lines[i]) >= 4) {
                code.push(lines[i].get(4..).unwrap_or("").to_string());
                i += 1;
            }
            while code.last().is_some_and(|line| blank(line)) {
                code.pop();
            }
            out.push(doc.leaf(BlockKind::code(""), vec![Inline::text(code.join("\n"))]));
        } else if let Some((ch, len, at, info)) = fence(line) {
            i += 1;
            let mut code = Vec::new();
            while i < lines.len() && !closes_fence(&lines[i], ch, len) {
                let strip = indent(&lines[i]).min(at);
                code.push(lines[i][strip..].to_string());
                i += 1;
            }
            i += 1;
            out.push(doc.leaf(BlockKind::code(info), vec![Inline::text(code.join("\n"))]));
        } else if let Some((level, text)) = atx(line) {
            out.push(doc.leaf(BlockKind::heading(level), inlines(&text)));
            i += 1;
        } else if rule(line) {
            out.push(doc.leaf(BlockKind::Rule, Vec::new()));
            i += 1;
        } else if quote_rest(line).is_some() {
            let mut inner: Vec<String> = Vec::new();
            while i < lines.len() {
                match quote_rest(&lines[i]) {
                    Some(rest) => inner.push(rest),
                    None if !blank(&lines[i]) && in_paragraph(&inner) && !interrupts(&lines[i]) => {
                        inner.push(lines[i].clone())
                    }
                    None => break,
                }
                i += 1;
            }
            let children = blocks(doc, &inner);
            out.push(doc.container(BlockKind::Quote, children));
        } else if let Some(first) = marker(line) {
            let same = |m: &Marker| m.ordered == first.ordered && m.ch == first.ch;
            let mut items = Vec::new();
            while let Some(item) = lines.get(i).and_then(|line| marker(line)).filter(same) {
                let mut inner = vec![item.rest.clone()];
                i += 1;
                while i < lines.len() {
                    let line = &lines[i];
                    if blank(line) {
                        inner.push(String::new());
                    } else if indent(line) >= item.width {
                        inner.push(line[item.width..].to_string());
                    } else if in_paragraph(&inner) && !interrupts(line) && marker(line).is_none() {
                        inner.push(line.trim_start().to_string());
                    } else {
                        break;
                    }
                    i += 1;
                }
                while inner.len() > 1 && inner.last().is_some_and(|line| blank(line)) {
                    inner.pop();
                }
                let children = blocks(doc, &inner);
                items.push(doc.container(BlockKind::ListItem, children));
            }
            let kind = match first.ordered {
                true => BlockKind::ordered_list(first.start),
                false => BlockKind::bullet_list(),
            };
            out.push(doc.container(kind, items));
        } else {
            let mut text = vec![line.trim_start().to_string()];
            let mut level = None;
            i += 1;
            while i < lines.len() && !blank(&lines[i]) {
                if let Some(found) = setext(&lines[i]) {
                    level = Some(found);
                    i += 1;
                    break;
                }
                if interrupts(&lines[i]) {
                    break;
                }
                text.push(lines[i].trim_start().to_string());
                i += 1;
            }
            let kind = level.map_or(BlockKind::Paragraph, BlockKind::heading);
            out.push(doc.leaf(
                kind.clone(),
                coerce(&kind, inlines(text.join("\n").trim_end())),
            ));
        }
    }
    out
}

enum Tok {
    Text(String),
    Code(String),
    Break,
    /// An emphasis or strike delimiter run with `count` chars left to match.
    Delim {
        ch: char,
        count: usize,
        length: usize,
        open: bool,
        close: bool,
    },
    Bracket {
        image: bool,
        active: bool,
    },
    /// An index into [`TAG_MARKS`], opening or closing.
    Tag(usize, bool),
}

struct Node {
    tok: Tok,
    marks: Marks,
}

/// The only HTML read as marks; any other tag stays literal text.
const TAG_MARKS: [(&str, Mark); 4] = [
    ("u", Mark::Underline),
    ("strong", Mark::Bold),
    ("em", Mark::Italic),
    ("s", Mark::Strike),
];

fn punctuation(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace()
}

fn node(tok: Tok) -> Node {
    Node {
        tok,
        marks: Marks::new(),
    }
}

fn inlines(text: &str) -> Vec<Inline> {
    let chars: Vec<char> = text.chars().collect();
    let mut nodes: Vec<Node> = Vec::new();
    let mut buf = String::new();
    let flush = |nodes: &mut Vec<Node>, buf: &mut String| {
        if !buf.is_empty() {
            nodes.push(node(Tok::Text(std::mem::take(buf))));
        }
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\\' if chars.get(i + 1) == Some(&'\n') => {
                flush(&mut nodes, &mut buf);
                nodes.push(node(Tok::Break));
                i += 2;
            }
            '\\' if chars.get(i + 1).is_some_and(char::is_ascii_punctuation) => {
                buf.push(chars[i + 1]);
                i += 2;
            }
            '`' => {
                let n = run(&chars, i, '`');
                match find_run(&chars, i + n, '`', n) {
                    Some(end) => {
                        flush(&mut nodes, &mut buf);
                        let code: String = chars[i + n..end]
                            .iter()
                            .map(|c| if *c == '\n' { ' ' } else { *c })
                            .collect();
                        let strip = code.len() >= 2
                            && code.starts_with(' ')
                            && code.ends_with(' ')
                            && !code.trim().is_empty();
                        let code = if strip {
                            code[1..code.len() - 1].to_string()
                        } else {
                            code
                        };
                        nodes.push(node(Tok::Code(code)));
                        i = end + n;
                    }
                    None => {
                        buf.extend(&chars[i..i + n]);
                        i += n;
                    }
                }
            }
            '<' => match angle(&chars, i) {
                Some((found, end)) => {
                    flush(&mut nodes, &mut buf);
                    nodes.push(found);
                    i = end;
                }
                None => {
                    buf.push('<');
                    i += 1;
                }
            },
            '!' if chars.get(i + 1) == Some(&'[') => {
                flush(&mut nodes, &mut buf);
                nodes.push(node(Tok::Bracket {
                    image: true,
                    active: true,
                }));
                i += 2;
            }
            '[' => {
                flush(&mut nodes, &mut buf);
                nodes.push(node(Tok::Bracket {
                    image: false,
                    active: true,
                }));
                i += 1;
            }
            ']' => {
                flush(&mut nodes, &mut buf);
                match close_bracket(&mut nodes, &chars, i) {
                    Some(end) => i = end,
                    None => {
                        buf.push(']');
                        i += 1;
                    }
                }
            }
            '*' | '_' | '~' => {
                let n = run(&chars, i, c);
                let before = if i == 0 { ' ' } else { chars[i - 1] };
                let after = chars.get(i + n).copied().unwrap_or(' ');
                let left = !after.is_whitespace()
                    && (!punctuation(after) || before.is_whitespace() || punctuation(before));
                let right = !before.is_whitespace()
                    && (!punctuation(before) || after.is_whitespace() || punctuation(after));
                let (open, close) = match c {
                    '_' => (
                        left && (!right || punctuation(before)),
                        right && (!left || punctuation(after)),
                    ),
                    _ => (left, right),
                };
                flush(&mut nodes, &mut buf);
                let usable = c != '~' || n <= 2;
                nodes.push(node(Tok::Delim {
                    ch: c,
                    count: n,
                    length: n,
                    open: open && usable,
                    close: close && usable,
                }));
                i += n;
            }
            '&' => match entity(&chars, i) {
                Some((decoded, end)) => {
                    buf.push(decoded);
                    i = end;
                }
                None => {
                    buf.push('&');
                    i += 1;
                }
            },
            '\n' => {
                let trimmed = buf.trim_end_matches(' ').len();
                let hard = buf.len() - trimmed >= 2;
                buf.truncate(trimmed);
                match hard {
                    true => {
                        flush(&mut nodes, &mut buf);
                        nodes.push(node(Tok::Break));
                    }
                    false => buf.push(' '),
                }
                i += 1;
            }
            _ => {
                buf.push(c);
                i += 1;
            }
        }
    }
    flush(&mut nodes, &mut buf);
    emphasis(&mut nodes, 0);
    finish(nodes)
}

fn run(chars: &[char], at: usize, c: char) -> usize {
    chars[at..].iter().take_while(|x| **x == c).count()
}

/// The start of the next run of exactly `n` `c`s.
fn find_run(chars: &[char], from: usize, c: char, n: usize) -> Option<usize> {
    let mut i = from;
    while i < chars.len() {
        let len = run(chars, i, c);
        if len == n {
            return Some(i);
        }
        i += len.max(1);
    }
    None
}

/// `<u>`-style tags and autolinks.
fn angle(chars: &[char], at: usize) -> Option<(Node, usize)> {
    let end = at + chars[at..].iter().position(|c| *c == '>')?;
    let inner: String = chars[at + 1..end].iter().collect();
    if inner.contains(['<', '\n', ' ']) {
        return None;
    }
    let (name, opening) = match inner.strip_prefix('/') {
        Some(name) => (name, false),
        None => (inner.as_str(), true),
    };
    if let Some(index) = TAG_MARKS
        .iter()
        .position(|(tag, _)| name.eq_ignore_ascii_case(tag))
    {
        return Some((node(Tok::Tag(index, opening)), end + 1));
    }
    let scheme = inner.split(':').next().unwrap_or("");
    let uri = inner.contains(':')
        && (2..=32).contains(&scheme.len())
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-'));
    let email = !uri
        && inner
            .split_once('@')
            .is_some_and(|(user, host)| !user.is_empty() && host.contains('.'));
    let href = match (uri, email) {
        (true, _) => inner.clone(),
        (_, true) => format!("mailto:{inner}"),
        _ => return None,
    };
    let mut found = node(Tok::Text(inner));
    if let Ok(href) = Href::parse(&href) {
        found.marks.add(Mark::Link { href, title: None });
    }
    Some((found, end + 1))
}

fn entity(chars: &[char], at: usize) -> Option<(char, usize)> {
    let end = at + chars[at..].iter().take(12).position(|c| *c == ';')?;
    let name: String = chars[at + 1..end].iter().collect();
    let decoded = match name.as_str() {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        "copy" => '©',
        _ => {
            let number = name.strip_prefix('#')?;
            let code = match number.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).ok()?,
                None => number.parse().ok()?,
            };
            char::from_u32(code)
                .filter(|c| *c != '\0')
                .unwrap_or('\u{fffd}')
        }
    };
    Some((decoded, end + 1))
}

/// A `]`: closes the nearest bracket into a link (or an image, kept as its alt text)
/// when an inline destination follows. Returns the index after it.
fn close_bracket(nodes: &mut [Node], chars: &[char], at: usize) -> Option<usize> {
    let open = nodes
        .iter()
        .rposition(|n| matches!(n.tok, Tok::Bracket { .. }))?;
    let Tok::Bracket { image, active } = nodes[open].tok else {
        unreachable!("a bracket")
    };
    let destination = (chars.get(at + 1) == Some(&'('))
        .then(|| destination(chars, at + 2))
        .flatten();
    let Some((href, title, end)) = destination.filter(|_| active) else {
        let literal = if image { "![" } else { "[" };
        nodes[open].tok = Tok::Text(literal.to_string());
        return None;
    };
    emphasis(nodes, open + 1);
    nodes[open].tok = Tok::Text(String::new());
    if !image {
        if let Ok(href) = Href::parse(&href) {
            let link = Mark::Link { href, title };
            for node in &mut nodes[open + 1..] {
                node.marks.add(link.clone());
            }
        }
        for node in &mut nodes[..open] {
            if let Tok::Bracket {
                image: false,
                active,
            } = &mut node.tok
            {
                *active = false;
            }
        }
    }
    Some(end)
}

/// `dest "title")` after `(`: the destination, the title and the index after `)`.
fn destination(chars: &[char], mut i: usize) -> Option<(String, Option<String>, usize)> {
    let skip = |i: &mut usize| {
        while chars.get(*i).is_some_and(|c| c.is_whitespace()) {
            *i += 1;
        }
    };
    skip(&mut i);
    let mut href = String::new();
    if chars.get(i) == Some(&'<') {
        i += 1;
        loop {
            match *chars.get(i)? {
                '>' => break,
                '\n' | '<' => return None,
                '\\' if chars.get(i + 1).is_some_and(char::is_ascii_punctuation) => {
                    href.push(chars[i + 1]);
                    i += 1;
                }
                c => href.push(c),
            }
            i += 1;
        }
        i += 1;
    } else {
        let mut depth = 0;
        while let Some(&c) = chars.get(i) {
            match c {
                '\\' if chars.get(i + 1).is_some_and(char::is_ascii_punctuation) => {
                    href.push(chars[i + 1]);
                    i += 1;
                }
                '(' => {
                    depth += 1;
                    href.push(c);
                }
                ')' if depth == 0 => break,
                ')' => {
                    depth -= 1;
                    href.push(c);
                }
                c if c.is_whitespace() || c.is_control() => break,
                c => href.push(c),
            }
            i += 1;
        }
    }
    skip(&mut i);
    let mut title = None;
    if let Some(&quote) = chars.get(i).filter(|c| matches!(c, '"' | '\'' | '(')) {
        let close = if quote == '(' { ')' } else { quote };
        let mut text = String::new();
        i += 1;
        loop {
            match *chars.get(i)? {
                c if c == close => break,
                '\\' if chars.get(i + 1).is_some_and(char::is_ascii_punctuation) => {
                    text.push(chars[i + 1]);
                    i += 1;
                }
                c => text.push(c),
            }
            i += 1;
        }
        i += 1;
        title = Some(text);
        skip(&mut i);
    }
    (chars.get(i) == Some(&')')).then_some((href, title, i + 1))
}

/// CommonMark's delimiter matching, over the nodes after `bottom`.
fn emphasis(nodes: &mut [Node], bottom: usize) {
    let mut closer = bottom;
    while closer < nodes.len() {
        let Tok::Delim {
            ch,
            count,
            length,
            close: true,
            open: closer_open,
        } = nodes[closer].tok
        else {
            closer += 1;
            continue;
        };
        if count == 0 {
            closer += 1;
            continue;
        }
        let opener = (bottom..closer).rev().find(|&o| match nodes[o].tok {
            Tok::Delim {
                ch: oc,
                count: on,
                length: ol,
                open: true,
                close: opener_close,
            } => {
                let both = opener_close || closer_open;
                oc == ch
                    && on > 0
                    && (ch != '~' || on == count)
                    && !(both && (ol + length) % 3 == 0 && !(ol % 3 == 0 && length % 3 == 0))
            }
            _ => false,
        });
        let Some(opener) = opener else {
            if !closer_open && let Tok::Delim { close, .. } = &mut nodes[closer].tok {
                *close = false;
            }
            closer += 1;
            continue;
        };
        let Tok::Delim {
            count: open_count, ..
        } = nodes[opener].tok
        else {
            unreachable!("a delimiter")
        };
        let used = match ch {
            '~' => count,
            _ if count >= 2 && open_count >= 2 => 2,
            _ => 1,
        };
        let mark = match (ch, used) {
            ('~', _) => Mark::Strike,
            (_, 2) => Mark::Bold,
            _ => Mark::Italic,
        };
        for node in &mut nodes[opener + 1..closer] {
            node.marks.add(mark.clone());
            if let Tok::Delim { open, close, .. } = &mut node.tok {
                *open = false;
                *close = false;
            }
        }
        for at in [opener, closer] {
            if let Tok::Delim { count, .. } = &mut nodes[at].tok {
                *count -= used;
            }
        }
    }
}

fn finish(nodes: Vec<Node>) -> Vec<Inline> {
    let mut open = [0usize; TAG_MARKS.len()];
    let mut out = Vec::new();
    for Node { tok, mut marks } in nodes {
        for ((_, mark), count) in TAG_MARKS.iter().zip(open) {
            if count > 0 {
                marks.add(mark.clone());
            }
        }
        match tok {
            Tok::Text(text) => out.push(Inline::marked(text, marks)),
            Tok::Code(code) => out.push(Inline::marked(code, marks.with(Mark::Code))),
            Tok::Break => out.push(Inline::HardBreak),
            Tok::Delim { ch, count, .. } => {
                out.push(Inline::marked(ch.to_string().repeat(count), marks))
            }
            Tok::Bracket { image, .. } => {
                out.push(Inline::marked(if image { "![" } else { "[" }, marks))
            }
            Tok::Tag(index, true) => open[index] += 1,
            Tok::Tag(index, false) => open[index] = open[index].saturating_sub(1),
        }
    }
    super::doc::normalize_inlines(&mut out);
    out
}
