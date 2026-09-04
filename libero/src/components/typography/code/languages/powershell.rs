//! Small hand-port of PowerShell's common syntax.

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
            PatternDef::new(r"'[^']*'").greedy(),
        ],
    },
    TokenRule {
        name: "variable",
        patterns: &[PatternDef::new(r"\$[a-zA-Z_][\w:]*").greedy()],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:function|param|if|elseif|else|foreach|for|while|do|until|switch|break|continue|return|try|catch|finally|throw|begin|process|end|in)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"\b[A-Z][a-zA-Z]*-[A-Z][a-zA-Z]*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b\d+(?:\.\d+)?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
