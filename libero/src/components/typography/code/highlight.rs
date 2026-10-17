//! A small port of Prism's (<https://prismjs.com>) tokenizer. Instead of Prism's greedy
//! rematch, adjacent greedy patterns compete by position.

use crate::components::common::Input;
use crate::localization::CodeBlockLabels;
use crate::platform::{PreparedText, RegexMatch, regex_api};

use super::language_catalog::{LANGUAGE_CATALOG, PLAIN_TEXT};

/// A highlighting language, parsed from a name such as `"rust"`. Each needs its `code-lang-*` feature.
#[derive(Clone, Copy)]
pub struct Language(&'static super::language_catalog::LanguageEntry);

impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for Language {}

impl std::fmt::Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Language").field(&self.0.label).finish()
    }
}

impl Language {
    fn parse(value: &str) -> Option<Self> {
        let value = value.to_lowercase();
        LANGUAGE_CATALOG
            .iter()
            .find(|entry| entry.aliases.contains(&value.as_str()))
            .map(Language)
    }

    /// The shown label; only plain text is localized.
    pub(crate) fn name(self, labels: &CodeBlockLabels) -> &'static str {
        if std::ptr::eq(self.0, &LANGUAGE_CATALOG[PLAIN_TEXT]) {
            labels.plain_text
        } else {
            self.0.label
        }
    }

    /// The compiled-in languages after plain text: label and the name a fence uses.
    pub(crate) fn catalog() -> impl Iterator<Item = (&'static str, &'static str)> {
        LANGUAGE_CATALOG
            .iter()
            .skip(PLAIN_TEXT + 1)
            .map(|entry| (entry.label, entry.aliases[0]))
    }

    /// The label of a fence's language name, `None` when none is compiled in.
    pub(crate) fn label_of(value: &str) -> Option<&'static str> {
        Self::parse(value).map(|language| language.0.label)
    }

    fn grammar(self) -> Grammar {
        (self.0.grammar)()
    }
}

/// Unrecognized input means "don't highlight", not "guess a language". An empty name, as
/// a plain fence gives, names no language, so it does not warn.
impl From<&str> for Input<Language> {
    fn from(value: &str) -> Self {
        if value.trim().is_empty() {
            return Input::None;
        }
        match Language::parse(value) {
            Some(language) => Input::Value(language),
            None => {
                crate::utils::warn(&format!(
                    "Code: unrecognized language {value:?}, rendering without highlighting. \
                     Its `code-lang-*` feature may be off."
                ));
                Input::None
            }
        }
    }
}

impl From<String> for Input<Language> {
    fn from(value: String) -> Self {
        Input::from(value.as_str())
    }
}

/// One alternative pattern for a [`TokenRule`], e.g.
/// `PatternDef::new(r"//.*").greedy()`.
pub(crate) struct PatternDef {
    pattern: &'static str,
    case_insensitive: bool,
    /// Lookbehind stand-in: matched, then stripped off the token's start (Prism's convention).
    lookbehind_group: Option<usize>,
    /// Lookahead stand-in: the token ends where this group starts.
    lookahead_group: Option<usize>,
    /// Adjacent greedy patterns match in one pass, leftmost first, ties to the earlier one:
    /// no comment starts inside a string, nor a string inside a comment.
    greedy: bool,
    /// Nesting stand-in: the token runs from the opener to its balancing closer, or the end.
    balanced: Option<(&'static str, &'static str)>,
    inside: Option<fn() -> Grammar>,
    alias: Option<&'static str>,
}

// A narrow `code-lang-*` feature set leaves some builders unused.
#[allow(dead_code)]
impl PatternDef {
    pub(crate) const fn new(pattern: &'static str) -> Self {
        Self {
            pattern,
            case_insensitive: false,
            lookbehind_group: None,
            lookahead_group: None,
            greedy: false,
            balanced: None,
            inside: None,
            alias: None,
        }
    }

    pub(crate) const fn case_insensitive(mut self) -> Self {
        self.case_insensitive = true;
        self
    }

    pub(crate) const fn lookbehind(mut self, group: usize) -> Self {
        self.lookbehind_group = Some(group);
        self
    }

    pub(crate) const fn lookahead(mut self, group: usize) -> Self {
        self.lookahead_group = Some(group);
        self
    }

    pub(crate) const fn greedy(mut self) -> Self {
        self.greedy = true;
        self
    }

    pub(crate) const fn balanced(mut self, open: &'static str, close: &'static str) -> Self {
        assert!(
            open.is_ascii() && close.is_ascii(),
            "`balanced_end` scans bytes"
        );
        self.balanced = Some((open, close));
        self
    }

    pub(crate) const fn inside(mut self, grammar: fn() -> Grammar) -> Self {
        self.inside = Some(grammar);
        self
    }

    pub(crate) const fn alias(mut self, alias: &'static str) -> Self {
        self.alias = Some(alias);
        self
    }
}

/// A token type and its patterns, tried in order: the first to match wins.
pub(crate) struct TokenRule {
    pub(crate) name: &'static str,
    pub(crate) patterns: &'static [PatternDef],
}

/// Priority is array order: earlier rules claim text before later ones see it.
pub(crate) type Grammar = &'static [TokenRule];

enum Token<'a> {
    Plain(&'a str),
    Tagged {
        name: &'static str,
        alias: Option<&'static str>,
        children: Vec<Token<'a>>,
    },
}

fn tokenize(text: &str, grammar: Grammar) -> Vec<Token<'_>> {
    let mut tokens = vec![Token::Plain(text)];
    let mut greedy_run: Vec<(&'static str, &PatternDef)> = Vec::new();
    for rule in grammar {
        for pattern in rule.patterns {
            if pattern.greedy {
                greedy_run.push((rule.name, pattern));
                continue;
            }
            if !greedy_run.is_empty() {
                apply_patterns(&mut tokens, &greedy_run);
                greedy_run.clear();
            }
            apply_patterns(&mut tokens, &[(rule.name, pattern)]);
        }
    }
    if !greedy_run.is_empty() {
        apply_patterns(&mut tokens, &greedy_run);
    }
    tokens
}

/// Tags every match of `patterns` in the `Plain` spans. Leftmost match wins, ties to the
/// earlier pattern; a match that started inside the winner is searched again after it.
fn apply_patterns<'a>(tokens: &mut Vec<Token<'a>>, patterns: &[(&'static str, &PatternDef)]) {
    // Rebuilt, not spliced: splicing is quadratic over a long block.
    let mut out = Vec::with_capacity(tokens.len());
    // Each pattern's next match; searched again only once the cursor passes it.
    let mut next: Vec<Option<RegexMatch>> = Vec::with_capacity(patterns.len());
    for token in tokens.drain(..) {
        let Token::Plain(span) = token else {
            out.push(token);
            continue;
        };

        // Prepared once per span: a fresh `&str` per `find` re-marshals the tail on wasm.
        let prepared = PreparedText::new(span);
        let mut cursor = 0;
        let find = |pattern: &PatternDef, cursor: usize| {
            regex_api().find(pattern.pattern, pattern.case_insensitive, &prepared, cursor)
        };
        next.clear();
        next.extend(patterns.iter().map(|(_, pattern)| find(pattern, 0)));

        loop {
            for (slot, (_, pattern)) in next.iter_mut().zip(patterns) {
                if slot.as_ref().is_some_and(|matched| matched.start < cursor) {
                    *slot = find(pattern, cursor);
                }
            }
            // `min_by_key` keeps the first of equal keys: ties go to the earlier pattern.
            let Some((index, matched)) = next
                .iter()
                .enumerate()
                .filter_map(|(index, slot)| slot.as_ref().map(|matched| (index, matched)))
                .min_by_key(|(_, matched)| matched.start)
            else {
                break;
            };
            let (name, pattern) = patterns[index];
            let (start, mut end) = resolve_span(pattern, matched);
            if let Some((open, close)) = pattern.balanced {
                end = balanced_end(span, end, open, close);
            }
            if start >= end {
                // An empty match tags nothing, but the pattern may still match further on.
                let width = span[matched.start..]
                    .chars()
                    .next()
                    .map_or(0, char::len_utf8);
                next[index] = (width > 0)
                    .then(|| find(pattern, matched.start + width))
                    .flatten();
                continue;
            }

            if start > cursor {
                out.push(Token::Plain(&span[cursor..start]));
            }
            let matched_text = &span[start..end];
            let children = match pattern.inside {
                Some(inside) => tokenize(matched_text, inside()),
                None => vec![Token::Plain(matched_text)],
            };
            out.push(Token::Tagged {
                name,
                alias: pattern.alias,
                children,
            });
            cursor = end;
        }

        if cursor < span.len() {
            out.push(Token::Plain(&span[cursor..]));
        }
    }
    *tokens = out;
}

/// The match span without its lookbehind prefix and lookahead suffix.
fn resolve_span(pattern: &PatternDef, matched: &RegexMatch) -> (usize, usize) {
    let start = pattern
        .lookbehind_group
        .and_then(|group| matched.group(group))
        .map(|(_, group_end)| group_end)
        .unwrap_or(matched.start);
    let end = pattern
        .lookahead_group
        .and_then(|group| matched.group(group))
        .map(|(group_start, _)| group_start)
        .unwrap_or(matched.end);
    (start, end)
}

/// The end of the `close` balancing an `open` that ended at `from`; `text.len()` if unclosed.
/// Bytewise: both delimiters are ASCII.
fn balanced_end(text: &str, from: usize, open: &str, close: &str) -> usize {
    let bytes = text.as_bytes();
    let mut depth = 1;
    let mut cursor = from;
    while cursor < bytes.len() {
        let rest = &bytes[cursor..];
        if rest.starts_with(close.as_bytes()) {
            depth -= 1;
            cursor += close.len();
            if depth == 0 {
                return cursor;
            }
        } else if rest.starts_with(open.as_bytes()) {
            depth += 1;
            cursor += open.len();
        } else {
            cursor += 1;
        }
    }
    text.len()
}

/// Prism token name/alias to `lsx-tok-*` class. Anything unmatched stays unstyled.
const TOKEN_CLASSES: &[(&str, &str)] = &[
    ("comment", "lsx-tok-comment"),
    ("string", "lsx-tok-string"),
    ("char", "lsx-tok-string"),
    ("attr-value", "lsx-tok-string"),
    ("code", "lsx-tok-string"),
    ("url", "lsx-tok-string"),
    ("number", "lsx-tok-number"),
    ("boolean", "lsx-tok-constant"),
    ("constant", "lsx-tok-constant"),
    ("variable", "lsx-tok-constant"),
    ("keyword", "lsx-tok-keyword"),
    ("builtin", "lsx-tok-keyword"),
    ("important", "lsx-tok-keyword"),
    ("function", "lsx-tok-function"),
    ("macro", "lsx-tok-function"),
    ("class-name", "lsx-tok-type"),
    ("tag", "lsx-tok-tag"),
    ("attr-name", "lsx-tok-attribute"),
    ("property", "lsx-tok-attribute"),
    ("selector", "lsx-tok-tag"),
    ("title", "lsx-tok-heading"),
    ("bold", "lsx-tok-bold"),
    ("italic", "lsx-tok-italic"),
];

fn classify_name(name: &str) -> Option<&'static str> {
    TOKEN_CLASSES
        .iter()
        .find(|(candidate, _)| *candidate == name)
        .map(|(_, class)| *class)
}

pub(crate) type HighlightedLine = Vec<(String, Option<&'static str>)>;

/// Flattens a token tree into `(text, class)` spans. A known own class (`alias` first)
/// beats the inherited one.
fn flatten<'a>(
    tokens: &[Token<'a>],
    out: &mut Vec<(&'a str, Option<&'static str>)>,
    inherited: Option<&'static str>,
) {
    for token in tokens {
        match token {
            Token::Plain(text) => {
                if !text.is_empty() {
                    out.push((text, inherited));
                }
            }
            Token::Tagged {
                name,
                alias,
                children,
            } => {
                let class = alias
                    .and_then(classify_name)
                    .or_else(|| classify_name(name))
                    .or(inherited);
                flatten(children, out, class);
            }
        }
    }
}

/// The last few highlights: a remount or a demo switching back skips the regex work (todo 2148).
const HIGHLIGHT_CACHE_LEN: usize = 32;

type HighlightCache = std::collections::VecDeque<(Language, String, Vec<HighlightedLine>)>;

thread_local! {
    static HIGHLIGHTED: std::cell::RefCell<HighlightCache> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}

pub(crate) fn highlight(source: &str, language: Language) -> Vec<HighlightedLine> {
    if source.is_empty() {
        return Vec::new();
    }
    let hit = HIGHLIGHTED.with_borrow(|cache| {
        cache
            .iter()
            .find(|(lang, text, _)| *lang == language && text == source)
            .map(|(_, _, lines)| lines.clone())
    });
    if let Some(lines) = hit {
        return lines;
    }

    let tokens = tokenize(source, language.grammar());
    let mut spans = Vec::new();
    flatten(&tokens, &mut spans, None);
    let lines = split_into_lines(spans, source.ends_with('\n'));
    HIGHLIGHTED.with_borrow_mut(|cache| {
        if cache.len() == HIGHLIGHT_CACHE_LEN {
            cache.pop_front();
        }
        cache.push_back((language, source.to_string(), lines.clone()));
    });
    lines
}

/// The only place a span's text is copied; `flatten`'s spans borrow the source.
fn split_into_lines(
    spans: Vec<(&str, Option<&'static str>)>,
    source_ends_with_newline: bool,
) -> Vec<HighlightedLine> {
    let mut lines: Vec<HighlightedLine> = vec![Vec::new()];

    for (text, class) in spans {
        let mut rest = text;
        while let Some(newline_pos) = rest.find('\n') {
            let line_part = rest[..newline_pos]
                .strip_suffix('\r')
                .unwrap_or(&rest[..newline_pos]);
            if !line_part.is_empty() {
                lines
                    .last_mut()
                    .expect("always at least one line")
                    .push((line_part.to_string(), class));
            }
            lines.push(Vec::new());
            rest = &rest[newline_pos + 1..];
        }
        if !rest.is_empty() {
            lines
                .last_mut()
                .expect("always at least one line")
                .push((rest.to_string(), class));
        }
    }

    // No trailing empty line from a trailing newline, matching `str::lines()`.
    if source_ends_with_newline && lines.last().is_some_and(Vec::is_empty) {
        lines.pop();
    }

    lines
}

#[cfg(test)]
mod tests;
