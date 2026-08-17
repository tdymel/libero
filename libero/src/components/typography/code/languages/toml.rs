//! Small hand-port of TOML's common syntax.
//!
//! Same "key anywhere before `:`... `=`" simplification as `yaml.rs`.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[PatternDef::new(r"#.*")],
    },
    TokenRule {
        name: "title",
        patterns: &[PatternDef::new(r"\[[\w.\s\[\]-]+\]")],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#""""[\s\S]*?""""#).greedy(),
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'[^']*'").greedy(),
        ],
    },
    TokenRule {
        name: "property",
        patterns: &[PatternDef::new(r"([\w.-]+)(\s*=)").lookahead(2)],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false)\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b-?\d[\d_]*(?:\.\d+)?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
