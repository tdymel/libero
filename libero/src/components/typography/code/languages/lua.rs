//! Small hand-port of Lua's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[
            PatternDef::new(r"--\[\[[\s\S]*?\]\]").greedy(),
            PatternDef::new(r"--.*").greedy(),
        ],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r"\[\[[\s\S]*?\]\]").greedy(),
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'(?:[^'\\]|\\.)*'").greedy(),
        ],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|nil)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:function|local|end|if|then|else|elseif|for|while|repeat|until|do|return|break|in|and|or|not|require)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_]\w*)(\s*\()").lookahead(2)],
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
