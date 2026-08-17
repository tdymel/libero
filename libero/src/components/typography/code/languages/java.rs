//! Small hand-port of Java's common syntax.

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
        name: "annotation",
        patterns: &[PatternDef::new(r"@[a-zA-Z_][\w.]*").alias("function")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|null)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:public|private|protected|class|interface|extends|implements|static|final|abstract|void|new|return|if|else|for|while|do|switch|case|default|break|continue|try|catch|finally|throw|throws|import|package|this|super|instanceof|synchronized|volatile|transient|native|enum|assert|record)\b",
        )],
    },
    TokenRule {
        name: "function",
        patterns: &[PatternDef::new(r"([a-zA-Z_]\w*)(\s*\()").lookahead(2)],
    },
    TokenRule {
        name: "class-name",
        patterns: &[PatternDef::new(r"\b[A-Z]\w*\b")],
    },
    TokenRule {
        name: "number",
        patterns: &[PatternDef::new(
            r"\b0[xX][\da-fA-F]+\b|\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?[lLfFdD]?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
