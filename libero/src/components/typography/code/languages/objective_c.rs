//! Small hand-port of Objective-C's common syntax.

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
        patterns: &[PatternDef::new(r#"@?"(?:[^"\\]|\\.)*""#).greedy()],
    },
    TokenRule {
        name: "macro",
        patterns: &[PatternDef::new(r"#\s*[a-zA-Z]+.*")],
    },
    TokenRule {
        name: "directive",
        patterns: &[PatternDef::new(r"@[a-zA-Z]+").alias("keyword")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:YES|NO|nil|NULL)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:if|else|for|while|do|switch|case|default|break|continue|return|self|super|static|const|extern|void|int|float|double|BOOL|id|typedef|struct|enum)\b",
        )],
    },
    super::FUNCTION_CALL,
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z]\w*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b\d+(?:\.\d+)?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
