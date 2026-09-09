//! Small hand-port of Kotlin's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[
            PatternDef::new(r"//.*").greedy(),
            // Kotlin's block comments nest.
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
        name: "annotation",
        patterns: &[PatternDef::new(r"@[A-Za-z_]\w*").alias("function")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|null)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:fun|val|var|if|else|when|for|while|do|return|break|continue|class|object|interface|package|import|is|in|as|this|super|try|catch|finally|throw|companion|init|constructor|override|open|abstract|final|private|protected|public|internal|lateinit|inline|suspend|typealias|data|sealed|enum)\b",
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
            r"\b0[xX][\da-fA-F]+\b|\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?[LlFf]?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
