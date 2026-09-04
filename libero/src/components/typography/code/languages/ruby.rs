//! Small hand-port of Ruby's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[PatternDef::new(r"#.*").greedy()],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'(?:[^'\\]|\\.)*'").greedy(),
        ],
    },
    TokenRule {
        name: "symbol",
        patterns: &[PatternDef::new(r":[a-zA-Z_]\w*[?!]?").alias("constant")],
    },
    TokenRule {
        name: "variable",
        patterns: &[PatternDef::new(r"@{1,2}[a-zA-Z_]\w*").greedy()],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|nil)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:def|end|class|module|if|elsif|else|unless|while|until|for|in|do|begin|rescue|ensure|raise|return|yield|require|require_relative|include|extend|attr_accessor|attr_reader|attr_writer|new|self|and|or|not|then|case|when|break|next|redo|retry|lambda|proc)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_]\w*[?!]?)(\s*\()").lookahead(2)],
    },
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z]\w*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(
            r"\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
