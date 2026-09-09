//! Small hand-port of Python's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[PatternDef::new(r"#.*").greedy()],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#"[a-zA-Z]?"""[\s\S]*?""""#).greedy(),
            PatternDef::new(r"[a-zA-Z]?'''[\s\S]*?'''").greedy(),
            PatternDef::new(r#"[a-zA-Z]?"(?:[^"\\\n]|\\.)*""#).greedy(),
            PatternDef::new(r"[a-zA-Z]?'(?:[^'\\\n]|\\.)*'").greedy(),
        ],
    },
    TokenRule {
        name: "decorator",
        patterns: &[PatternDef::new(r"@[a-zA-Z_][\w.]*").alias("function")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:True|False|None)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:def|class|return|if|elif|else|for|while|break|continue|pass|import|from|as|with|try|except|finally|raise|lambda|global|nonlocal|yield|assert|del|in|is|not|and|or|async|await)\b",
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
            r"\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?j?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
