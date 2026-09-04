//! Small hand-port of C++'s common syntax.

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
        name: "macro",
        patterns: &[PatternDef::new(r"#\s*[a-zA-Z]+.*")],
    },
    TokenRule {
        name: "boolean",
        patterns: &[PatternDef::new(r"\b(?:true|false|nullptr)\b")],
    },
    TokenRule {
        name: "keyword",
        patterns: &[PatternDef::new(
            r"\b(?:auto|break|case|char|class|const|constexpr|continue|default|delete|do|double|else|enum|explicit|extern|float|for|friend|goto|if|inline|int|long|mutable|namespace|new|noexcept|operator|override|private|protected|public|register|return|short|signed|sizeof|static|struct|switch|template|this|throw|try|catch|typedef|typename|union|unsigned|using|virtual|void|volatile|while)\b",
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
            r"\b0[xX][\da-fA-F]+\b|\b\d+(?:\.\d+)?(?:[eE][+-]?\d+)?[uUlLfF]*\b",
        )],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
