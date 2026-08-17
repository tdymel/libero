//! Small port of https://github.com/PrismJS/prism/blob/v2/src/languages/prism-rust.js

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
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'(?:[^'\\]|\\.)'").greedy().alias("char"),
        ],
    },
    TokenRule {
        name: "macro",
        patterns: &[PatternDef::new(r"\b[a-zA-Z_][a-zA-Z0-9_]*!").greedy()],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:as|async|await|break|const|continue|crate|dyn|else|enum|extern|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|Self|static|struct|super|trait|type|unsafe|use|where|while)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_][a-zA-Z0-9_]*)(\s*\()").lookahead(2)],
    },
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z][a-zA-Z0-9_]*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(
            r"\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?(?:u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize|f32|f64)?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
