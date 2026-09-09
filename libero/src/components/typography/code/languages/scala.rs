//! Small hand-port of Scala's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[
            PatternDef::new(r"//.*").greedy(),
            // Scala's block comments nest.
            PatternDef::new(r"/\*").balanced("/*", "*/").greedy(),
        ],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#""""[\s\S]*?""""#).greedy(),
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
        ],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|null)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:def|val|var|if|else|for|while|do|match|case|class|object|trait|extends|with|import|package|new|this|super|return|yield|try|catch|finally|throw|implicit|override|abstract|final|private|protected|sealed|lazy)\b",
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
            r"\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?[LlFfDd]?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
