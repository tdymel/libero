//! The Markdown subset's recognizers, shared by the typing shortcuts ([`super::rules`])
//! and the reader ([`super::parse`]), so both accept exactly the same syntax.

use super::mark::Mark;

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
