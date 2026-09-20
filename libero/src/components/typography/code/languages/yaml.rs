//! Small hand-port of YAML's common syntax. `key` is any identifier before `:`, not only
//! at line start: the engine has no per-line anchors.

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
        name: "property",
        patterns: &[PatternDef::new(r"([\w.-]+)(\s*:)").lookahead(2)],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|yes|no|on|off|null|~)\b").case_insensitive()],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b-?\d+(?:\.\d+)?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
