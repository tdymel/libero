//! Small port of https://github.com/PrismJS/prism/blob/v2/src/languages/prism-bash.js

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[PatternDef::new(r"#.*")],
    },
    TokenRule {
        name: "string",
        patterns: &[
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'[^']*'").greedy(),
        ],
    },
    TokenRule {
        name: "variable",
        patterns: &[PatternDef::new(r"\$\{?[a-zA-Z_][a-zA-Z0-9_]*\}?").greedy()],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:if|then|elif|else|fi|for|while|until|do|done|case|esac|function|in|select|time|return|break|continue|local|export|readonly|declare|unset)\b",
        )],
    },
    TokenRule {
        name: "builtin",
        patterns: &[PatternDef::new(
            r"\b(?:cd|echo|pwd|exit|set|shift|eval|exec|trap|wait|read|printf|test|source|alias|unalias|cat|grep|sed|awk|find|ls|mkdir|rm|cp|mv|chmod|chown|curl|wget|ssh|git|docker)\b",
        )],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(r"\b\d+\b")],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
