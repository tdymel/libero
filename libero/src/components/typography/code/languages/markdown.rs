//! Small port of https://github.com/PrismJS/prism/blob/v2/src/languages/prism-markdown.js

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "code",
        patterns: &[PatternDef::new(r"`[^`\n]+`").greedy()],
    },
    TokenRule {
        name: "title",
        // `.` doesn't match `\n` by default, so this naturally stops at the
        // end of the heading's own line without needing a `^`/`$` anchor.
        patterns: &[PatternDef::new(r"(#{1,6}[ \t]+).+").lookbehind(1)],
    },
    TokenRule {
        name: "bold",
        patterns: &[
            PatternDef::new(r"(\*\*)(.+?)(\*\*)")
                .lookbehind(1)
                .lookahead(3)
                .greedy(),
            PatternDef::new(r"(__)(.+?)(__)")
                .lookbehind(1)
                .lookahead(3)
                .greedy(),
        ],
    },
    TokenRule {
        name: "italic",
        patterns: &[
            PatternDef::new(r"(\*)(.+?)(\*)")
                .lookbehind(1)
                .lookahead(3)
                .greedy(),
            PatternDef::new(r"(_)(.+?)(_)")
                .lookbehind(1)
                .lookahead(3)
                .greedy(),
        ],
    },
    TokenRule {
        name: "url",
        patterns: &[PatternDef::new(r"\[[^\]\n]*\]\([^)\n]*\)").greedy()],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
