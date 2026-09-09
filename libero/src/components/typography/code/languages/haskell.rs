//! Small hand-port of Haskell's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[
            PatternDef::new(r"--.*").greedy(),
            // Haskell's block comments nest.
            PatternDef::new(r"\{-").balanced("{-", "-}").greedy(),
        ],
    },
    TokenRule {
        name: "string",
        patterns: &[PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy()],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:module|import|where|let|in|if|then|else|case|of|do|data|type|newtype|class|instance|deriving|infix|infixl|infixr)\b",
        )],
    },
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z]\w*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
