//! Small port of https://github.com/PrismJS/prism/blob/v2/src/languages/prism-css.js
//! `property` is any identifier before `:`, so an unquoted `word:` in `url(...)` misfires.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[PatternDef::new(r"/\*[\s\S]*?\*/").greedy()],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#""(?:[^"\\\n]|\\.)*""#).greedy(),
            PatternDef::new(r"'(?:[^'\\\n]|\\.)*'").greedy(),
        ],
    },
    TokenRule {
        name: "atrule",
        patterns: &[PatternDef::new(r"@[-\w]+").alias("keyword")],
    },
    TokenRule {
        name: "important",
        patterns: &[PatternDef::new(r"!\s*important").case_insensitive()],
    },
    TokenRule {
        name: "property",
        patterns: &[PatternDef::new(r"([-\w]+)(\s*:)").lookahead(2)],
    },
    TokenRule {
        name: "constant",
        patterns: &[
            PatternDef::new(
                r"\b(?:red|orange|yellow|green|blue|indigo|violet|black|white|gray|grey|transparent|currentColor)\b",
            ),
            PatternDef::new(r"#[0-9a-fA-F]{3,8}\b"),
        ],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"-?\d*\.?\d+(?:%|[a-zA-Z]+)?")],
    },
    TokenRule {
        name: "selector",
        patterns: &[PatternDef::new(r"([.#]?[-\w]+)(\s*[,{])").lookahead(2)],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
