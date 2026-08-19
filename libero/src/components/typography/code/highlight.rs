//! A small port of Prism (https://prismjs.com)'s tokenizing engine.
//!
//! Simplified: matching only happens within text no higher-priority rule has
//! claimed, dropping Prism's rarer greedy rematch across already-tokenized
//! siblings. The hand-ported grammars in `languages/` don't need it.

#[cfg(feature = "wasm-split")]
use dioxus::wasm_split;

use crate::components::Input;
use crate::components::common::{RegexMatch, regex_api};

use super::language_catalog::LANGUAGE_CATALOG;

/// Always has a grammar: the catalog only lists languages compiled in by
/// their `code-lang-*` feature.
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

    pub(crate) fn label(self) -> &'static str {
        self.0.label
    }

    fn grammar(self) -> Grammar {
        (self.0.grammar)()
    }
}

/// Unrecognized input means "don't highlight", not "guess a language".
impl From<&str> for Input<Language> {
    fn from(value: &str) -> Self {
        match Language::parse(value) {
            Some(language) => Input::Value(language),
            None => {
                crate::utils::warn(&format!(
                    "Code: unrecognized language {value:?}, rendering without highlighting.                      Its `code-lang-*` feature may be off."
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
    /// Lookbehind stand-in: matched, then stripped off the token's start.
    /// Prism's own convention, so neither engine needs real lookbehind.
    lookbehind_group: Option<usize>,
    /// Lookahead stand-in: matched so the pattern can assert on what follows
    /// (a CSS property's trailing `:`), but the token ends where this group
    /// starts rather than at the match's end.
    lookahead_group: Option<usize>,
    /// No effect here - this engine only matches untokenized text. Kept for
    /// parity when cross-referencing upstream Prism grammars.
    #[allow(dead_code)]
    greedy: bool,
    inside: Option<fn() -> Grammar>,
    alias: Option<&'static str>,
}

impl PatternDef {
    pub(crate) const fn new(pattern: &'static str) -> Self {
        Self {
            pattern,
            case_insensitive: false,
            lookbehind_group: None,
            lookahead_group: None,
            greedy: false,
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

    pub(crate) const fn inside(mut self, grammar: fn() -> Grammar) -> Self {
        self.inside = Some(grammar);
        self
    }

    pub(crate) const fn alias(mut self, alias: &'static str) -> Self {
        self.alias = Some(alias);
        self
    }
}

/// A token type and its patterns, tried in array order - Prism's "first
/// pattern to match wins".
pub(crate) struct TokenRule {
    pub(crate) name: &'static str,
    pub(crate) patterns: &'static [PatternDef],
}

/// Priority is array order: earlier rules claim text before later ones see
/// it, as Prism does with object-key insertion order.
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
    for rule in grammar {
        for pattern in rule.patterns {
            apply_pattern(&mut tokens, rule.name, pattern);
        }
    }
    tokens
}

/// Replaces every match of `pattern` within still-untokenized (`Plain`)
/// spans with a `Tagged` token, recursing into `inside` grammars first.
fn apply_pattern<'a>(tokens: &mut Vec<Token<'a>>, name: &'static str, pattern: &PatternDef) {
    let mut index = 0;
    while index < tokens.len() {
        let Token::Plain(text) = tokens[index] else {
            index += 1;
            continue;
        };

        let Some(matched) = regex_api().find(pattern.pattern, pattern.case_insensitive, text)
        else {
            index += 1;
            continue;
        };

        let (start, end) = resolve_span(pattern, &matched);
        if start >= end {
            index += 1;
            continue;
        }

        let before = &text[..start];
        let matched_text = &text[start..end];
        let after = &text[end..];

        let children = match pattern.inside {
            Some(inside) => tokenize(matched_text, inside()),
            None => vec![Token::Plain(matched_text)],
        };
        let tagged = Token::Tagged {
            name,
            alias: pattern.alias,
            children,
        };

        let mut replacement = Vec::with_capacity(3);
        if !before.is_empty() {
            replacement.push(Token::Plain(before));
        }
        replacement.push(tagged);
        if !after.is_empty() {
            replacement.push(Token::Plain(after));
        }

        let has_after = !after.is_empty();
        let inserted = replacement.len();
        tokens.splice(index..=index, replacement);
        // Skip `before` (it can hold no earlier match - `find` returns the
        // leftmost) and the new token, but land on `after` so later matches
        // of the same pattern are still found.
        index += inserted - if has_after { 1 } else { 0 };
    }
}

/// Strips a lookbehind group's prefix and/or a lookahead group's suffix out
/// of the match span.
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

/// Prism token name/alias -> the `lsx-tok-*` classes `token_theme.rs` styles.
/// Deliberately not exhaustive; anything unmatched stays unstyled.
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

/// Flattens a token tree into `(text, class)` spans. A token's own class beats
/// its parent's when it maps to a known one (`alias` first, being more
/// specific); otherwise it inherits its ancestor's.
fn flatten(
    tokens: &[Token<'_>],
    out: &mut Vec<(String, Option<&'static str>)>,
    inherited: Option<&'static str>,
) {
    for token in tokens {
        match token {
            Token::Plain(text) => {
                if !text.is_empty() {
                    out.push((text.to_string(), inherited));
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

/// [`highlight`]'s shape with no language, so `Code` keeps one render path.
pub(crate) fn plain_lines(source: &str) -> Vec<HighlightedLine> {
    source
        .lines()
        .map(|line| vec![(line.to_string(), None)])
        .collect()
}

pub(crate) fn highlight(source: &str, language: Language) -> Vec<HighlightedLine> {
    if source.is_empty() {
        return Vec::new();
    }

    let tokens = tokenize(source, language.grammar());
    let mut spans = Vec::new();
    flatten(&tokens, &mut spans, None);
    split_into_lines(spans, source.ends_with('\n'))
}

fn split_into_lines(
    spans: Vec<(String, Option<&'static str>)>,
    source_ends_with_newline: bool,
) -> Vec<HighlightedLine> {
    let mut lines: Vec<HighlightedLine> = vec![Vec::new()];

    for (text, class) in spans {
        let mut rest = text.as_str();
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

    // No trailing empty line from a trailing newline, matching `str::lines()`
    // and therefore `plain_lines`.
    if source_ends_with_newline && lines.last().is_some_and(Vec::is_empty) {
        lines.pop();
    }

    lines
}

// `#[wasm_split]` rebuilds the fn from its signature alone and so drops
// visibility on wasm32; `highlight_lazy` re-exposes it.
#[cfg(feature = "wasm-split")]
#[wasm_split::wasm_split(code_highlighting)]
async fn highlight_split(source: String, language: Language) -> Vec<HighlightedLine> {
    highlight(&source, language)
}

/// [`highlight`], but under the `wasm-split` feature the engine lives in a
/// separate chunk fetched on first call. Without it, a plain async call.
pub(crate) async fn highlight_lazy(source: String, language: Language) -> Vec<HighlightedLine> {
    #[cfg(feature = "wasm-split")]
    return highlight_split(source, language).await;

    #[cfg(not(feature = "wasm-split"))]
    return highlight(&source, language);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lang(name: &str) -> Language {
        Language::parse(name).unwrap_or_else(|| panic!("{name} should be in LANGUAGE_CATALOG"))
    }

    /// Every grammar must tokenize arbitrary text without panicking or
    /// hanging - `apply_pattern`'s "start >= end" guard is what stops a
    /// pathological pattern looping forever.
    #[test]
    fn every_catalog_language_highlights_without_panicking() {
        let sample = "hello_world(123) // a comment \"a string\" 4.5 { } [ ] < > = : ; \n";
        for entry in LANGUAGE_CATALOG.iter() {
            let language = Language(entry);
            let lines = highlight(sample, language);
            assert!(!lines.is_empty(), "{} produced no lines", entry.label);
        }
    }

    fn flat(source: &str, language: Language) -> Vec<(String, Option<&'static str>)> {
        highlight(source, language).into_iter().flatten().collect()
    }

    #[test]
    fn language_parse_accepts_known_aliases_and_rejects_unknown() {
        assert_eq!(Language::parse("rust"), Language::parse("rs"));
        assert_eq!(Language::parse("rust"), Language::parse("RS"));
        assert_eq!(Language::parse("bash"), Language::parse("sh"));
        assert_eq!(Language::parse("md"), Language::parse("markdown"));
        assert_eq!(Language::parse("cobol"), None);
    }

    #[test]
    fn unrecognized_language_string_falls_back_to_no_highlighting() {
        let input: Input<Language> = "cobol".into();
        assert!(matches!(input, Input::None));
    }

    #[test]
    fn highlight_rust_classifies_keyword_string_comment_and_number() {
        let spans = flat("let x = 5; // hi\n", lang("rust"));

        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "let" && *class == Some("lsx-tok-keyword"))
        );
        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "5" && *class == Some("lsx-tok-number"))
        );
        assert!(
            spans
                .iter()
                .any(|(text, class)| text.contains("hi") && *class == Some("lsx-tok-comment"))
        );
    }

    #[test]
    fn highlight_rust_string_quotes_share_the_string_class() {
        let spans = flat("\"hi\"\n", lang("rust"));

        assert!(
            spans
                .iter()
                .all(|(text, class)| text.trim().is_empty() || *class == Some("lsx-tok-string"))
        );
    }

    #[test]
    fn highlight_html_classifies_tag_and_attribute() {
        let spans = flat("<div class=\"a\"></div>\n", lang("html"));

        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "div" && *class == Some("lsx-tok-tag"))
        );
        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "class" && *class == Some("lsx-tok-attribute"))
        );
        // Both the opening and closing tag name must be classified, not just
        // the first occurrence in the source.
        assert_eq!(
            spans
                .iter()
                .filter(|(text, class)| *text == "div" && *class == Some("lsx-tok-tag"))
                .count(),
            2
        );
    }

    #[test]
    fn highlight_css_classifies_property_and_color_constant() {
        let spans = flat("a { color: red; }\n", lang("css"));

        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "color" && *class == Some("lsx-tok-attribute"))
        );
        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "red" && *class == Some("lsx-tok-constant"))
        );
    }

    #[test]
    fn highlight_shell_classifies_keyword_and_string() {
        let spans = flat("echo \"hi\"\n", lang("bash"));

        assert!(
            spans
                .iter()
                .any(|(text, class)| text.contains("hi") && *class == Some("lsx-tok-string"))
        );
        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "echo" && *class == Some("lsx-tok-keyword"))
        );
    }

    #[test]
    fn highlight_markdown_classifies_heading_bold_and_italic() {
        let spans = flat("# Title\n\n*a* **b**\n", lang("markdown"));

        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "Title" && *class == Some("lsx-tok-heading"))
        );
        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "a" && *class == Some("lsx-tok-italic"))
        );
        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "b" && *class == Some("lsx-tok-bold"))
        );
    }

    #[test]
    fn highlight_never_includes_trailing_newline_in_span_text() {
        let spans = flat("let x = 1;\n", lang("rust"));
        assert!(spans.iter().all(|(text, _)| !text.contains('\n')));
    }

    #[test]
    fn highlight_drops_trailing_empty_line_but_keeps_interior_blank_lines() {
        let source = "let a = 1;\n\nlet b = 2;\n";
        let lines = highlight(source, lang("rust"));
        assert_eq!(lines.len(), 3);
        assert!(lines[1].iter().all(|(text, _)| text.is_empty()));
    }
}
