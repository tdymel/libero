//! Small hand-port of JavaScript's common syntax.

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
            PatternDef::new(r"'(?:[^'\\]|\\.)*'").greedy(),
            PatternDef::new(r"`(?:[^`\\]|\\.)*`").greedy(),
        ],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|null|undefined)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:var|let|const|function|return|if|else|for|while|do|switch|case|default|break|continue|class|extends|new|this|super|import|export|from|as|try|catch|finally|throw|typeof|instanceof|in|of|async|await|yield|static|get|set|delete|void)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_$][\w$]*)(\s*\()").lookahead(2)],
    },
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z][\w$]*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(
            r"\b0[xX][\da-fA-F]+\b|\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
