//! The Markdown subset's recognizers, shared by the typing shortcuts ([`super::rules`])
//! and the reader ([`super::parse`]), so both accept exactly the same syntax.

use super::mark::{Href, Mark};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BlockSyntax {
    Heading(u8),
    /// The bullet char: `-`, `*` or `+`.
    Bullet(char),
    /// The start number and the delimiter after it: `.` or `)`.
    Ordered(u64, char),
    Quote,
    Rule,
    /// The backtick count and the info string.
    Fence(usize, String),
}

/// What a block prefix means: the whole prefix, without the space that ends it.
pub(super) fn block_syntax(prefix: &str) -> Option<BlockSyntax> {
    let fence = prefix.chars().take_while(|c| *c == '`').count();
    if fence >= 3 && !prefix[fence..].contains('`') {
        return Some(BlockSyntax::Fence(
            fence,
            prefix[fence..].trim().to_string(),
        ));
    }
    if !prefix.is_empty() && prefix.len() <= 6 && prefix.chars().all(|c| c == '#') {
        return Some(BlockSyntax::Heading(prefix.len() as u8));
    }
    match prefix {
        "-" | "*" | "+" => prefix.chars().next().map(BlockSyntax::Bullet),
        ">" => Some(BlockSyntax::Quote),
        "---" | "***" | "___" => Some(BlockSyntax::Rule),
        number => {
            let delimiter = number.chars().last().filter(|c| matches!(c, '.' | ')'))?;
            let digits = &number[..number.len() - 1];
            if !(1..=9).contains(&digits.len()) || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            Some(BlockSyntax::Ordered(digits.parse().ok()?, delimiter))
        }
    }
}

/// The span delimiters, code first: a code span shields its inside from the others.
pub(super) const DELIMITERS: [(&str, Mark); 4] = [
    ("`", Mark::Code),
    ("**", Mark::Bold),
    ("~~", Mark::Strike),
    ("*", Mark::Italic),
];

/// Whether `inner` may sit between two `delimiter`s: not empty, no edge space or
/// delimiter char, no break or node, and no backtick inside code.
pub(super) fn valid_inner(inner: &[char], delimiter: &str) -> bool {
    let first = delimiter.chars().next();
    let edge = |c: Option<&char>| c.is_none_or(|c| Some(*c) == first || c.is_whitespace());
    !edge(inner.first())
        && !edge(inner.last())
        && !(first == Some('`') && inner.contains(&'`'))
        && !inner.iter().any(|c| matches!(c, '\n' | '\u{fffc}'))
}

/// A span that `text`'s tail closes: the opener's index, the delimiter length and the mark.
/// `active` says which chars may act as delimiters (not backslash-escaped ones).
pub(super) fn closing_span(
    text: &[char],
    active: impl Fn(usize) -> bool,
    delimiters: &[(&str, Mark)],
) -> Option<(usize, usize, Mark)> {
    for (delimiter, mark) in delimiters {
        let delimiter: Vec<char> = delimiter.chars().collect();
        let len = delimiter.len();
        if text.len() < 2 * len + 1 || !text.ends_with(&delimiter) {
            continue;
        }
        let close = text.len() - len;
        let live = |at: usize| (at..at + len).all(&active);
        // `*` must not be the tail of `**`.
        if !live(close) || len == 1 && delimiter[0] == '*' && text[close - 1] == '*' {
            continue;
        }
        let Some(open) = (0..close.saturating_sub(len))
            .rev()
            .find(|start| text[*start..*start + len] == delimiter[..] && live(*start))
        else {
            continue;
        };
        let inner = &text[open + len..close];
        let doubled = len == 1 && delimiter[0] == '*' && open > 0 && text[open - 1] == '*';
        if !doubled && valid_inner(inner, &delimiter.iter().collect::<String>()) {
            return Some((open, len, mark.clone()));
        }
    }
    None
}

/// The URL a typed or pasted word starts with: its length in chars, without the
/// punctuation that ends a sentence, and its href. Only the [`Href::SCHEMES`].
pub(super) fn autolink(word: &[char]) -> Option<(usize, Href)> {
    if word.iter().any(|c| c.is_whitespace()) {
        return None;
    }
    let lower = word.iter().collect::<String>().to_ascii_lowercase();
    let head = Href::SCHEMES.iter().find_map(|scheme| {
        let head = match *scheme {
            "mailto" => "mailto:".to_string(),
            scheme => format!("{scheme}://"),
        };
        lower.starts_with(&head).then(|| head.chars().count())
    })?;
    let mut end = word.len();
    while end > head {
        let count = |c: char| word[..end].iter().filter(|x| **x == c).count();
        let trailing = word[end - 1];
        if matches!(trailing, '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"')
            || trailing == ')' && count(')') > count('(')
        {
            end -= 1;
        } else {
            break;
        }
    }
    let rest = &word[head..end];
    let valid = match lower.starts_with("mailto:") {
        true => rest
            .iter()
            .position(|c| *c == '@')
            .is_some_and(|at| at > 0 && at + 1 < rest.len()),
        false => rest.first().is_some_and(|c| c.is_alphanumeric()),
    };
    let href = Href::parse(&word[..end].iter().collect::<String>()).ok()?;
    valid.then_some((end, href))
}

/// A `[text](url "title")` link at `at` with a safe URL: its `]`, the index after `)`, the mark.
pub(super) fn link(chars: &[char], at: usize) -> Option<(usize, usize, Mark)> {
    let mut depth = 0;
    let mut close = at + 1;
    loop {
        match *chars.get(close)? {
            '\\' => close += 1,
            '[' => depth += 1,
            ']' if depth == 0 => break,
            ']' => depth -= 1,
            _ => {}
        }
        close += 1;
    }
    if chars.get(close + 1) != Some(&'(') {
        return None;
    }
    let (href, title, end) = destination(chars, close + 2)?;
    let href = Href::parse(&href).ok()?;
    Some((close, end, Mark::Link { href, title }))
}

/// `url "title")` after `(`: the URL, the title and the index after `)`.
fn destination(chars: &[char], mut i: usize) -> Option<(String, Option<String>, usize)> {
    let skip = |i: &mut usize| {
        while chars.get(*i) == Some(&' ') {
            *i += 1;
        }
    };
    // Reads up to `stop`, unescaping; nested parens only where `stop` is `)`.
    let read = |i: &mut usize, stop: fn(char) -> bool| -> Option<String> {
        let mut out = String::new();
        let mut depth = 0;
        loop {
            let mut c = *chars.get(*i)?;
            match c {
                '\\' if chars.get(*i + 1).is_some_and(char::is_ascii_punctuation) => {
                    *i += 1;
                    c = chars[*i];
                }
                '\n' => return None,
                ')' if depth > 0 => depth -= 1,
                c if stop(c) => return Some(out),
                '(' => depth += 1,
                _ => {}
            }
            out.push(c);
            *i += 1;
        }
    };
    skip(&mut i);
    let href = match chars.get(i) == Some(&'<') {
        true => {
            i += 1;
            let href = read(&mut i, |c| c == '>')?;
            i += 1;
            href
        }
        false => read(&mut i, |c| c == ')' || c == ' ')?,
    };
    skip(&mut i);
    let mut title = None;
    if chars.get(i) == Some(&'"') {
        i += 1;
        title = Some(read(&mut i, |c| c == '"')?);
        i += 1;
        skip(&mut i);
    }
    (chars.get(i) == Some(&')')).then_some((href, title, i + 1))
}
