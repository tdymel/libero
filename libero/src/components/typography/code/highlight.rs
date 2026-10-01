//! A small port of Prism's (<https://prismjs.com>) tokenizer. Instead of Prism's greedy
//! rematch, adjacent greedy patterns compete by position.

use crate::components::common::Input;
use crate::localization::CodeBlockLabels;
use crate::platform::{PreparedText, RegexMatch, regex_api};

use super::language_catalog::LANGUAGE_CATALOG;

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
        match std::ptr::eq(self.0, &LANGUAGE_CATALOG[0]) {
            true => labels.plain_text,
            false => self.0.label,
        }
    }

    /// The compiled-in languages after plain text: label and the name a fence uses.
    pub(crate) fn catalog() -> impl Iterator<Item = (&'static str, &'static str)> {
        LANGUAGE_CATALOG
            .iter()
            .skip(1)
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

/// Unrecognized input means "don't highlight", not "guess a language".
impl From<&str> for Input<Language> {
    fn from(value: &str) -> Self {
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
                next[index] = None;
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

pub(crate) fn highlight(source: &str, language: Language) -> Vec<HighlightedLine> {
    if source.is_empty() {
        return Vec::new();
    }

    let tokens = tokenize(source, language.grammar());
    let mut spans = Vec::new();
    flatten(&tokens, &mut spans, None);
    split_into_lines(spans, source.ends_with('\n'))
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
mod tests {
    use super::*;

    /// A test naming a language needs its feature's `cfg`, or `--no-default-features` panics here.
    fn lang(name: &str) -> Language {
        Language::parse(name).unwrap_or_else(|| panic!("{name} should be in LANGUAGE_CATALOG"))
    }

    /// Sweeps over every grammar; compiled out without one, where they would check nothing
    /// (review 7 S2). The batch gate runs them with all features on.
    #[cfg(any(
        feature = "code-lang-bash",
        feature = "code-lang-c",
        feature = "code-lang-cpp",
        feature = "code-lang-csharp",
        feature = "code-lang-css",
        feature = "code-lang-dart",
        feature = "code-lang-go",
        feature = "code-lang-graphql",
        feature = "code-lang-haskell",
        feature = "code-lang-html",
        feature = "code-lang-java",
        feature = "code-lang-javascript",
        feature = "code-lang-json",
        feature = "code-lang-kotlin",
        feature = "code-lang-lua",
        feature = "code-lang-markdown",
        feature = "code-lang-objective-c",
        feature = "code-lang-perl",
        feature = "code-lang-php",
        feature = "code-lang-powershell",
        feature = "code-lang-python",
        feature = "code-lang-r",
        feature = "code-lang-ruby",
        feature = "code-lang-rust",
        feature = "code-lang-scala",
        feature = "code-lang-sql",
        feature = "code-lang-swift",
        feature = "code-lang-toml",
        feature = "code-lang-typescript",
        feature = "code-lang-yaml",
    ))]
    mod grammar_sweeps {
        use super::*;

        /// No panic or hang: `apply_patterns`' `start >= end` guard stops an endless loop.
        #[test]
        fn every_catalog_language_highlights_without_panicking() {
            let sample = "hello_world(123) // a comment \"a string\" 4.5 { } [ ] < > = : ; \n";
            for entry in LANGUAGE_CATALOG.iter() {
                let language = Language(entry);
                let lines = highlight(sample, language);
                assert!(!lines.is_empty(), "{} produced no lines", entry.label);
            }
        }

        /// Todo 279: the native ASCII rewrite of `\b`, `\w`, `\d` must leave every pattern,
        /// nested ones included, compiling.
        #[test]
        fn every_grammar_pattern_still_compiles_after_the_ascii_rewrite() {
            fn compile(grammar: Grammar, label: &str, depth: usize) {
                assert!(
                    depth < 8,
                    "{label}: `inside` grammars nest suspiciously deep"
                );
                let text = PreparedText::new("");
                for rule in grammar {
                    for pattern in rule.patterns {
                        // A compile failure panics; `None` only means no match.
                        regex_api().find(pattern.pattern, pattern.case_insensitive, &text, 0);
                        if let Some(inside) = pattern.inside {
                            compile(inside(), label, depth + 1);
                        }
                    }
                }
            }

            for entry in LANGUAGE_CATALOG.iter() {
                compile((entry.grammar)(), entry.label, 0);
            }
        }

        /// Comment and string only compete when no non-greedy pattern sits between them.
        #[test]
        fn every_grammars_comment_and_string_rules_compete() {
            for entry in LANGUAGE_CATALOG.iter() {
                let patterns: Vec<(&str, bool)> = (entry.grammar)()
                    .iter()
                    .flat_map(|rule| rule.patterns.iter().map(|p| (rule.name, p.greedy)))
                    .collect();
                let delimited = |(name, _): &(&str, bool)| *name == "comment" || *name == "string";
                let (Some(first), Some(last)) = (
                    patterns.iter().position(delimited),
                    patterns.iter().rposition(delimited),
                ) else {
                    continue;
                };
                assert!(
                    patterns[first..=last].iter().all(|(_, greedy)| *greedy),
                    "{}: a non-greedy pattern splits its comment and string rules",
                    entry.label
                );
            }
        }
    }

    /// Review 6 S1: `é` is a word character to `regex` but not to `RegExp`; both engines
    /// must agree, or fullstack hydration mismatches.
    #[test]
    #[cfg(feature = "code-lang-rust")]
    fn a_keyword_after_a_non_ascii_letter_tokenizes_as_it_does_on_the_web() {
        let spans = flat("éif x; if y\n", lang("rust"));

        assert_eq!(
            spans,
            vec![
                ("é".to_string(), None),
                ("if".to_string(), Some("lsx-tok-keyword")),
                (" x; ".to_string(), None),
                ("if".to_string(), Some("lsx-tok-keyword")),
                (" y".to_string(), None),
            ]
        );
    }

    fn flat(source: &str, language: Language) -> Vec<(String, Option<&'static str>)> {
        highlight(source, language).into_iter().flatten().collect()
    }

    /// With a feature off, both sides are `None` and the test proves nothing.
    #[test]
    #[cfg(all(
        feature = "code-lang-rust",
        feature = "code-lang-bash",
        feature = "code-lang-markdown"
    ))]
    fn language_parse_accepts_known_aliases_and_rejects_unknown() {
        assert_eq!(Language::parse("rust"), Language::parse("rs"));
        assert_eq!(Language::parse("rust"), Language::parse("RS"));
        assert_eq!(Language::parse("bash"), Language::parse("sh"));
        assert_eq!(Language::parse("md"), Language::parse("markdown"));
        assert_eq!(Language::parse("cobol"), None);
    }

    #[test]
    fn plain_text_is_a_known_language_that_highlights_nothing() {
        for alias in ["text", "plain", "plaintext", "txt", "Text"] {
            let input: Input<Language> = alias.into();
            assert!(matches!(input, Input::Value(_)), "{alias} should parse");
        }
        assert_eq!(lang("text").name(&CodeBlockLabels::ENGLISH), "Plain text");
        assert_eq!(lang("text").name(&CodeBlockLabels::GERMAN), "Nur-Text");
        assert_eq!(
            flat("let x = \"//\"; // hi\n", lang("text")),
            vec![("let x = \"//\"; // hi".to_string(), None)]
        );
    }

    #[test]
    #[cfg(feature = "code-lang-rust")]
    fn highlight_rust_comment_marker_inside_a_string_stays_string() {
        let spans = flat("to: \"https://dioxuslabs.com\", // link\n", lang("rust"));

        assert!(spans.contains(&(
            "\"https://dioxuslabs.com\"".to_string(),
            Some("lsx-tok-string")
        )));
        assert!(spans.contains(&("// link".to_string(), Some("lsx-tok-comment"))));
    }

    #[test]
    #[cfg(feature = "code-lang-rust")]
    fn highlight_rust_quote_and_line_comment_inside_a_comment_stay_comment() {
        let spans = flat("// say \"hi\"\n/* a // b */ x\n", lang("rust"));

        assert!(spans.contains(&("// say \"hi\"".to_string(), Some("lsx-tok-comment"))));
        assert!(spans.contains(&("/* a // b */".to_string(), Some("lsx-tok-comment"))));
    }

    #[test]
    #[cfg(feature = "code-lang-rust")]
    fn highlight_rust_nested_block_comment_ends_at_its_balancing_close() {
        let spans = flat("/* a /* b */ c */ x /**/ \"/*\" y\n", lang("rust"));

        assert_eq!(
            spans,
            vec![
                ("/* a /* b */ c */".to_string(), Some("lsx-tok-comment")),
                (" x ".to_string(), None),
                ("/**/".to_string(), Some("lsx-tok-comment")),
                (" ".to_string(), None),
                ("\"/*\"".to_string(), Some("lsx-tok-string")),
                (" y".to_string(), None),
            ]
        );
    }

    #[cfg(any(
        feature = "code-lang-swift",
        feature = "code-lang-kotlin",
        feature = "code-lang-scala",
        feature = "code-lang-dart",
        feature = "code-lang-haskell"
    ))]
    fn assert_nested_comment(language: &str, source: &str, comment: &str, rest: &str) {
        assert_eq!(
            flat(source, lang(language)),
            vec![
                (comment.to_string(), Some("lsx-tok-comment")),
                (rest.to_string(), None),
            ],
            "{language}"
        );
    }

    #[test]
    #[cfg(feature = "code-lang-swift")]
    fn highlight_swift_nested_block_comment_ends_at_its_balancing_close() {
        assert_nested_comment("swift", "/* a /* b */ c */ x\n", "/* a /* b */ c */", " x");
    }

    #[test]
    #[cfg(feature = "code-lang-kotlin")]
    fn highlight_kotlin_nested_block_comment_ends_at_its_balancing_close() {
        assert_nested_comment("kotlin", "/* a /* b */ c */ x\n", "/* a /* b */ c */", " x");
    }

    #[test]
    #[cfg(feature = "code-lang-scala")]
    fn highlight_scala_nested_block_comment_ends_at_its_balancing_close() {
        assert_nested_comment("scala", "/* a /* b */ c */ x\n", "/* a /* b */ c */", " x");
    }

    #[test]
    #[cfg(feature = "code-lang-dart")]
    fn highlight_dart_nested_block_comment_ends_at_its_balancing_close() {
        assert_nested_comment("dart", "/* a /* b */ c */ x\n", "/* a /* b */ c */", " x");
    }

    #[test]
    #[cfg(feature = "code-lang-haskell")]
    fn highlight_haskell_nested_block_comment_ends_at_its_balancing_close() {
        assert_nested_comment(
            "haskell",
            "{- a {- b -} c -} x\n",
            "{- a {- b -} c -}",
            " x",
        );
    }

    #[test]
    #[cfg(feature = "code-lang-rust")]
    fn highlight_rust_unclosed_block_comment_runs_to_the_end() {
        let spans = flat("x /* a /* b */ é\ny\n", lang("rust"));

        assert_eq!(
            spans,
            vec![
                ("x ".to_string(), None),
                ("/* a /* b */ é".to_string(), Some("lsx-tok-comment")),
                ("y".to_string(), Some("lsx-tok-comment")),
            ]
        );
    }

    #[test]
    fn balanced_end_counts_nesting_like_a_lexer() {
        let end = |text: &str| balanced_end(text, 2, "/*", "*/");
        assert_eq!(end("/**/ x"), 4);
        assert_eq!(end("/*/ */ x"), 6);
        assert_eq!(end("/* /* */ */ x"), 11);
        assert_eq!(end("/* /* */"), 8);
    }

    #[test]
    #[cfg(feature = "code-lang-bash")]
    fn highlight_shell_hash_inside_a_string_stays_string() {
        let spans = flat("echo \"a#b\" # c\n", lang("bash"));

        assert!(spans.contains(&("\"a#b\"".to_string(), Some("lsx-tok-string"))));
        assert!(spans.contains(&("# c".to_string(), Some("lsx-tok-comment"))));
    }

    #[test]
    fn unrecognized_language_string_falls_back_to_no_highlighting() {
        let input: Input<Language> = "cobol".into();
        assert!(matches!(input, Input::None));
    }

    #[test]
    #[cfg(feature = "code-lang-rust")]
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
    #[cfg(feature = "code-lang-rust")]
    fn highlight_rust_string_quotes_share_the_string_class() {
        let spans = flat("\"hi\"\n", lang("rust"));

        // Every byte is drawn, so an empty result cannot pass the `all` below.
        assert_eq!(
            spans.iter().map(|(text, _)| &**text).collect::<String>(),
            "\"hi\""
        );
        assert!(
            spans
                .iter()
                .all(|(text, class)| text.trim().is_empty() || *class == Some("lsx-tok-string")),
            "{spans:?}"
        );
    }

    #[test]
    #[cfg(feature = "code-lang-html")]
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
        // Both the opening and the closing tag name.
        assert_eq!(
            spans
                .iter()
                .filter(|(text, class)| *text == "div" && *class == Some("lsx-tok-tag"))
                .count(),
            2
        );
    }

    #[test]
    #[cfg(feature = "code-lang-css")]
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
    #[cfg(feature = "code-lang-bash")]
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
    #[cfg(feature = "code-lang-markdown")]
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
    #[cfg(feature = "code-lang-rust")]
    fn highlight_never_includes_trailing_newline_in_span_text() {
        let spans = flat("let x = 1;\n", lang("rust"));
        assert_eq!(
            spans.iter().map(|(text, _)| &**text).collect::<String>(),
            "let x = 1;"
        );
        assert!(
            spans.iter().all(|(text, _)| !text.contains('\n')),
            "{spans:?}"
        );
    }

    #[test]
    #[cfg(feature = "code-lang-rust")]
    fn highlight_drops_trailing_empty_line_but_keeps_interior_blank_lines() {
        let source = "let a = 1;\n\nlet b = 2;\n";
        let lines = highlight(source, lang("rust"));
        assert_eq!(lines.len(), 3);
        assert!(lines[1].iter().all(|(text, _)| text.is_empty()));
    }
}
