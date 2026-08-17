//! Small hand-port of Perl's common syntax.

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
        name: "variable",
        patterns: &[PatternDef::new(r"[\$@%][a-zA-Z_]\w*").greedy()],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:my|our|local|sub|return|if|elsif|else|unless|while|until|for|foreach|do|last|next|redo|use|require|package|no|print|defined|undef|and|or|not|eq|ne|lt|gt|le|ge)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_]\w*)(\s*\()").lookahead(2)],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b\d+(?:\.\d+)?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
