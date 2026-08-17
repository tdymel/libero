//! Small hand-port of JSON's syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "property",
        patterns: &[PatternDef::new(r#"("(?:[^"\\]|\\.)*")(\s*:)"#).lookahead(2)],
    },
    TokenRule {
        name: "string",
        patterns: &[PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy()],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false)\b")],
    },
    TokenRule {
        name: "constant",
        patterns: &[PatternDef::new(r"\bnull\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
