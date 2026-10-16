use super::*;

/// A test naming a language needs its feature's `cfg`, or `--no-default-features` panics here.
fn lang(name: &str) -> Language {
    Language::parse(name).unwrap_or_else(|| panic!("{name} should be in LANGUAGE_CATALOG"))
}

fn flat(source: &str, language: Language) -> Vec<(String, Option<&'static str>)> {
    highlight(source, language).into_iter().flatten().collect()
}

/// Asserts a span with exactly `text` and `class`, printing all spans on failure.
// Unused when no language feature is compiled in.
#[allow(dead_code)]
fn assert_span(spans: &[(String, Option<&'static str>)], text: &str, class: &str) {
    assert!(
        spans
            .iter()
            .any(|(span, span_class)| span == text && *span_class == Some(class)),
        "no `{text}` with {class} in {spans:?}"
    );
}

/// Sweeps over every compiled-in grammar; with none they only see plain text.
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

    assert_span(&spans, "\"https://dioxuslabs.com\"", "lsx-tok-string");
    assert_span(&spans, "// link", "lsx-tok-comment");
}

#[test]
#[cfg(feature = "code-lang-rust")]
fn highlight_rust_quote_and_line_comment_inside_a_comment_stay_comment() {
    let spans = flat("// say \"hi\"\n/* a // b */ x\n", lang("rust"));

    assert_span(&spans, "// say \"hi\"", "lsx-tok-comment");
    assert_span(&spans, "/* a // b */", "lsx-tok-comment");
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

// Unused when no nested-comment language is compiled in.
#[allow(dead_code)]
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

    assert_span(&spans, "\"a#b\"", "lsx-tok-string");
    assert_span(&spans, "# c", "lsx-tok-comment");
}

#[test]
fn unrecognized_language_string_falls_back_to_no_highlighting() {
    let input: Input<Language> = "cobol".into();
    assert!(matches!(input, Input::None));
    assert_eq!(crate::utils::take_warnings().len(), 1);
}

#[test]
fn an_empty_language_is_none_without_a_warning() {
    for value in ["", "  "] {
        let input: Input<Language> = value.into();
        assert!(matches!(input, Input::None), "{value:?}");
    }
    assert_eq!(crate::utils::take_warnings(), Vec::<String>::new());
}

#[test]
#[cfg(feature = "code-lang-rust")]
fn highlight_rust_classifies_keyword_string_comment_and_number() {
    let spans = flat("let x = 5; // hi\n", lang("rust"));

    assert_span(&spans, "let", "lsx-tok-keyword");
    assert_span(&spans, "5", "lsx-tok-number");
    assert_span(&spans, "// hi", "lsx-tok-comment");
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

    assert_span(&spans, "div", "lsx-tok-tag");
    assert_span(&spans, "class", "lsx-tok-attribute");
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

    assert_span(&spans, "color", "lsx-tok-attribute");
    assert_span(&spans, "red", "lsx-tok-constant");
}

#[test]
#[cfg(feature = "code-lang-bash")]
fn highlight_shell_classifies_keyword_and_string() {
    let spans = flat("echo \"hi\"\n", lang("bash"));

    assert_span(&spans, "\"hi\"", "lsx-tok-string");
    assert_span(&spans, "echo", "lsx-tok-keyword");
}

#[test]
#[cfg(feature = "code-lang-markdown")]
fn highlight_markdown_classifies_heading_bold_and_italic() {
    let spans = flat("# Title\n\n*a* **b**\n", lang("markdown"));

    assert_span(&spans, "Title", "lsx-tok-heading");
    assert_span(&spans, "a", "lsx-tok-italic");
    assert_span(&spans, "b", "lsx-tok-bold");
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

#[test]
#[cfg(feature = "code-lang-rust")]
fn highlight_splits_crlf_lines_without_a_carriage_return_in_any_span() {
    let lines = highlight("let a = 1;\r\nlet b = 2;\r\n", lang("rust"));

    assert_eq!(lines.len(), 2);
    assert!(
        lines.iter().flatten().all(|(text, _)| !text.contains('\r')),
        "{lines:?}"
    );
}

#[test]
fn an_empty_match_does_not_hide_the_patterns_later_matches() {
    static GRAMMAR: Grammar = &[TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new("x*")],
    }];
    let tokens = tokenize("abxc", GRAMMAR);
    let mut spans = Vec::new();
    flatten(&tokens, &mut spans, None);

    assert_eq!(
        spans,
        [("ab", None), ("x", Some("lsx-tok-keyword")), ("c", None)]
    );
}
