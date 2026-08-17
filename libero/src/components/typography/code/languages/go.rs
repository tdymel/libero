//! Small hand-port of Go's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[
            PatternDef::new(r"//.*").greedy(),
            PatternDef::new(r"/\*[\s\S]*?\*/").greedy(),
        ],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"`[^`]*`").greedy(),
            PatternDef::new(r"'(?:[^'\\]|\\.)'").greedy().alias("char"),
        ],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|nil)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:package|import|func|return|if|else|for|range|switch|case|default|break|continue|go|defer|chan|select|var|const|type|struct|interface|map|goto|fallthrough)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_]\w*)(\s*\()").lookahead(2)],
    },
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z]\w*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(
            r"\b0[xX][\da-fA-F]+\b|\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
