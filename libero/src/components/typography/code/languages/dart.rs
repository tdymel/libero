//! Small hand-port of Dart's common syntax.

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
            PatternDef::new(r#""""[\s\S]*?""""#).greedy(),
            PatternDef::new(r#""(?:[^"\\]|\\.)*""#).greedy(),
            PatternDef::new(r"'(?:[^'\\]|\\.)*'").greedy(),
        ],
    },
    TokenRule {
        name: "annotation",
        patterns: &[PatternDef::new(r"@[A-Za-z_]\w*").alias("function")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|null)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:class|void|var|final|const|if|else|for|while|do|switch|case|default|break|continue|return|import|export|library|part|extends|implements|with|abstract|static|async|await|try|catch|finally|throw|new|this|super|is|as|typedef|enum|mixin|factory|get|set|late|required)\b",
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
            r"\b\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
