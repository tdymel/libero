//! Small hand-port of R's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[PatternDef::new(r"#.*")],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'(?:[^'\\]|\\.)*'").greedy(),
        ],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:TRUE|FALSE|NA|NULL|Inf|NaN)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:function|if|else|for|while|repeat|break|next|return|library|require|in)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_.][\w.]*)(\s*\()").lookahead(2)],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?[Li]?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
