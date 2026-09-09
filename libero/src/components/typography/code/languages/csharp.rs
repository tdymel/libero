//! Small hand-port of C#'s common syntax.

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
            PatternDef::new(r#"\$?@?"(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'(?:[^'\\]|\\.)'").greedy().alias("char"),
        ],
    },
    TokenRule {
        name: "attribute",
        patterns: &[PatternDef::new(r"\[[a-zA-Z_][\w.]*(?:\([^)]*\))?\]").alias("function")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|null)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:using|namespace|class|public|private|protected|internal|static|void|new|return|if|else|for|foreach|in|while|do|switch|case|default|break|continue|try|catch|finally|throw|async|await|this|base|override|virtual|abstract|sealed|readonly|const|struct|interface|enum|get|set|var|record)\b",
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
            r"\b0[xX][\da-fA-F]+\b|\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?[dDfFmMuUlL]?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
