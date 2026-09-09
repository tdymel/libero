//! Small hand-port of Swift's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[
            PatternDef::new(r"//.*").greedy(),
            // Swift's block comments nest.
            PatternDef::new(r"/\*").balanced("/*", "*/").greedy(),
        ],
    },
    TokenRule {
        name: "string",
        patterns: &[PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy()],
    },
    TokenRule {
        name: "attribute",
        patterns: &[PatternDef::new(r"@[A-Za-z_]\w*").alias("function")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|nil)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:func|var|let|if|else|guard|switch|case|default|for|in|while|repeat|do|catch|throw|throws|try|return|break|continue|class|struct|enum|protocol|extension|import|public|private|internal|fileprivate|static|final|override|init|deinit|self|super|as|is|where|typealias|associatedtype|mutating|convenience|lazy|weak|unowned|willSet|didSet)\b",
        )],
    },
    super::FUNCTION_CALL,
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z]\w*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(
            r"\b0[xX][\da-fA-F]+\b|\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
