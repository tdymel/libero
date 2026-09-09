//! Small hand-port of SQL's common syntax.

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[
            PatternDef::new(r"--.*").greedy(),
            PatternDef::new(r"/\*[\s\S]*?\*/").greedy(),
        ],
    },
    TokenRule {
        name: "string",
        patterns: &[PatternDef::new(r"'[^']*'").greedy()],
    },
    TokenRule {
        name: "keyword",
        patterns: &[
            PatternDef::new(
                r"\b(?:SELECT|FROM|WHERE|INSERT|INTO|VALUES|UPDATE|SET|DELETE|CREATE|TABLE|ALTER|DROP|JOIN|INNER|LEFT|RIGHT|OUTER|ON|GROUP|BY|ORDER|HAVING|LIMIT|OFFSET|UNION|ALL|DISTINCT|AS|AND|OR|NOT|NULL|IS|IN|LIKE|BETWEEN|EXISTS|CASE|WHEN|THEN|ELSE|END|PRIMARY|KEY|FOREIGN|REFERENCES|DEFAULT|CONSTRAINT|INDEX|VIEW|PROCEDURE|FUNCTION|TRIGGER|BEGIN|COMMIT|ROLLBACK|TRANSACTION)\b",
            )
            .case_insensitive(),
        ],
    },
    super::FUNCTION_CALL,
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b\d+(?:\.\d+)?\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
