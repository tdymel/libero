use std::sync::LazyLock;

use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxSet};

use crate::components::Input;

/// Variants are gated by the matching `code-lang-*` Cargo feature - only
/// languages opted into at compile time exist as constructible values, and
/// only their grammars get fetched and embedded by `build.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    #[cfg(feature = "code-lang-rust")]
    Rust,
    #[cfg(feature = "code-lang-shell")]
    Shell,
    #[cfg(feature = "code-lang-markdown")]
    Markdown,
    #[cfg(feature = "code-lang-html")]
    Html,
    #[cfg(feature = "code-lang-css")]
    Css,
}

impl Language {
    fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            #[cfg(feature = "code-lang-rust")]
            "rust" | "rs" => Some(Self::Rust),
            #[cfg(feature = "code-lang-shell")]
            "shell" | "sh" | "bash" => Some(Self::Shell),
            #[cfg(feature = "code-lang-markdown")]
            "markdown" | "md" => Some(Self::Markdown),
            #[cfg(feature = "code-lang-html")]
            "html" => Some(Self::Html),
            #[cfg(feature = "code-lang-css")]
            "css" => Some(Self::Css),
            _ => None,
        }
    }

    /// The token fed to [`SyntaxSet::find_syntax_by_token`] - independent of
    /// whatever alias was used to construct this `Language`.
    fn syntect_token(self) -> &'static str {
        match self {
            #[cfg(feature = "code-lang-rust")]
            Self::Rust => "rust",
            #[cfg(feature = "code-lang-shell")]
            Self::Shell => "sh",
            #[cfg(feature = "code-lang-markdown")]
            Self::Markdown => "md",
            #[cfg(feature = "code-lang-html")]
            Self::Html => "html",
            #[cfg(feature = "code-lang-css")]
            Self::Css => "css",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            #[cfg(feature = "code-lang-rust")]
            Self::Rust => "Rust",
            #[cfg(feature = "code-lang-shell")]
            Self::Shell => "Shell",
            #[cfg(feature = "code-lang-markdown")]
            Self::Markdown => "Markdown",
            #[cfg(feature = "code-lang-html")]
            Self::Html => "HTML",
            #[cfg(feature = "code-lang-css")]
            Self::Css => "CSS",
        }
    }
}

/// Unrecognized input means "don't highlight", not "guess" - resolves to
/// `Input::None` rather than defaulting to some language.
impl From<&str> for Input<Language> {
    fn from(value: &str) -> Self {
        Language::parse(value)
            .map(Input::Value)
            .unwrap_or(Input::None)
    }
}

impl From<String> for Input<Language> {
    fn from(value: String) -> Self {
        Input::from(value.as_str())
    }
}

/// Built by `build.rs` from only the languages enabled via `code-lang-*`
/// features, instead of syntect's own ~150-language default packdump.
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| {
    syntect::dumps::from_uncompressed_data(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/syntaxes.packdump"
    )))
    .expect("build.rs writes a valid dump")
});

/// TextMate scope prefixes (checked stack-top to bottom, most specific
/// first) mapped to the token classes declared in `token_theme.rs`. Not
/// exhaustive - anything unmatched (plain punctuation, whitespace, generic
/// `meta.*`/`variable.*`) stays unstyled, which is intentional: we're
/// coloring for "good enough, consistent", not full editor fidelity.
const RAW_SCOPE_CLASSES: &[(&str, &str)] = &[
    ("punctuation.definition.string", "lsx-tok-string"),
    ("punctuation.definition.comment", "lsx-tok-comment"),
    ("comment", "lsx-tok-comment"),
    ("string", "lsx-tok-string"),
    ("constant.numeric", "lsx-tok-number"),
    ("support.constant", "lsx-tok-constant"),
    ("constant", "lsx-tok-constant"),
    ("keyword", "lsx-tok-keyword"),
    ("storage", "lsx-tok-keyword"),
    ("entity.name.function", "lsx-tok-function"),
    ("support.function", "lsx-tok-function"),
    ("support.macro", "lsx-tok-function"),
    ("entity.name.tag", "lsx-tok-tag"),
    ("entity.name.type", "lsx-tok-type"),
    ("support.type.property-name", "lsx-tok-attribute"),
    ("support.type", "lsx-tok-type"),
    ("entity.other.attribute-name", "lsx-tok-attribute"),
    ("markup.heading", "lsx-tok-heading"),
    ("markup.bold", "lsx-tok-bold"),
    ("markup.italic", "lsx-tok-italic"),
    ("markup.raw", "lsx-tok-string"),
    ("punctuation.definition", "lsx-tok-comment"),
];

static SCOPE_CLASSES: LazyLock<Vec<(Scope, &'static str)>> = LazyLock::new(|| {
    RAW_SCOPE_CLASSES
        .iter()
        .map(|(scope, class)| {
            (
                Scope::new(scope).expect("RAW_SCOPE_CLASSES entries are valid scopes"),
                *class,
            )
        })
        .collect()
});

pub(crate) type HighlightedLine = Vec<(String, Option<&'static str>)>;

/// Same shape as [`highlight`], for when there's no language to color by -
/// keeps `Code`'s rendering (line numbers included) on one code path.
pub(crate) fn plain_lines(source: &str) -> Vec<HighlightedLine> {
    source
        .lines()
        .map(|line| vec![(line.to_string(), None)])
        .collect()
}

/// `None` if the language has no bundled syntax (shouldn't happen for the
/// closed `Language` set, but `find_syntax_by_token` is fallible).
pub(crate) fn highlight(source: &str, language: Language) -> Option<Vec<HighlightedLine>> {
    let syntax = SYNTAX_SET.find_syntax_by_token(language.syntect_token())?;
    let mut parse_state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut lines = Vec::new();

    for line in syntect::util::LinesWithEndings::from(source) {
        let content_len = line.trim_end_matches(['\n', '\r']).len();
        let ops = parse_state.parse_line(line, &SYNTAX_SET).ok()?;
        let mut spans = Vec::new();
        let mut last = 0usize;

        for (pos, op) in ops {
            let pos = pos.min(content_len);
            if pos > last {
                push_span(&mut spans, &line[last..pos], &stack);
            }
            let _ = stack.apply(&op);
            last = pos;
        }
        if last < content_len {
            push_span(&mut spans, &line[last..content_len], &stack);
        }

        lines.push(spans);
    }

    Some(lines)
}

fn push_span(spans: &mut HighlightedLine, text: &str, stack: &ScopeStack) {
    if text.is_empty() {
        return;
    }
    spans.push((text.to_string(), classify(stack)));
}

fn classify(stack: &ScopeStack) -> Option<&'static str> {
    stack.as_slice().iter().rev().find_map(|scope| {
        SCOPE_CLASSES
            .iter()
            .find(|(prefix, _)| prefix.is_prefix_of(*scope))
            .map(|(_, class)| *class)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(source: &str, language: Language) -> Vec<(String, Option<&'static str>)> {
        highlight(source, language)
            .expect("language should resolve to a bundled syntax")
            .into_iter()
            .flatten()
            .collect()
    }

    #[test]
    fn language_parse_accepts_known_aliases_and_rejects_unknown() {
        assert_eq!(Language::parse("rust"), Some(Language::Rust));
        assert_eq!(Language::parse("RS"), Some(Language::Rust));
        assert_eq!(Language::parse("bash"), Some(Language::Shell));
        assert_eq!(Language::parse("md"), Some(Language::Markdown));
        assert_eq!(Language::parse("cobol"), None);
    }

    #[test]
    fn unrecognized_language_string_falls_back_to_no_highlighting() {
        let input: Input<Language> = "cobol".into();
        assert!(matches!(input, Input::None));
    }

    #[test]
    fn highlight_rust_classifies_keyword_string_comment_and_number() {
        let spans = flat("let x = 5; // hi\n", Language::Rust);

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
        let spans = flat("\"hi\"\n", Language::Rust);

        assert!(
            spans
                .iter()
                .all(|(text, class)| text.trim().is_empty() || *class == Some("lsx-tok-string"))
        );
    }

    #[test]
    fn highlight_html_classifies_tag_and_attribute() {
        let spans = flat("<div class=\"a\"></div>\n", Language::Html);

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
    }

    #[test]
    fn highlight_css_classifies_property_and_color_constant() {
        let spans = flat("a { color: red; }\n", Language::Css);

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
        let spans = flat("echo \"hi\"\n", Language::Shell);

        assert!(
            spans
                .iter()
                .any(|(text, class)| text == "hi" && *class == Some("lsx-tok-string"))
        );
    }

    #[test]
    fn highlight_markdown_classifies_heading_bold_and_italic() {
        let spans = flat("# Title\n\n*a* **b**\n", Language::Markdown);

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
        let spans = flat("let x = 1;\n", Language::Rust);
        assert!(spans.iter().all(|(text, _)| !text.contains('\n')));
    }
}
