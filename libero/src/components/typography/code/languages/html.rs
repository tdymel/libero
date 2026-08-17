//! Small port of https://github.com/PrismJS/prism/blob/v2/src/languages/prism-markup.js
//! (Prism calls it "markup" - HTML/XML share one grammar there too).

use crate::components::typography::code::highlight::{Grammar, PatternDef, TokenRule};

const TAG_INSIDE: &[TokenRule] = &[
    TokenRule {
        name: "tag",
        patterns: &[PatternDef::new(r"(</?)([-\w]+)").lookbehind(1)],
    },
    TokenRule {
        name: "attr-value",
        patterns: &[
            PatternDef::new(r#"(=\s*)"[^"]*""#).lookbehind(1).greedy(),
            PatternDef::new(r"(=\s*)'[^']*'").lookbehind(1).greedy(),
        ],
    },
    TokenRule {
        name: "attr-name",
        patterns: &[PatternDef::new(r"([-\w]+)(\s*=)").lookahead(2)],
    },
    TokenRule {
        name: "punctuation",
        patterns: &[PatternDef::new(r"/?>|</?|=")],
    },
];

fn tag_inside() -> Grammar {
    TAG_INSIDE
}

const RULES: &[TokenRule] = &[
    TokenRule {
        name: "comment",
        patterns: &[PatternDef::new(r"<!--[\s\S]*?-->").greedy()],
    },
    TokenRule {
        name: "tag",
        patterns: &[PatternDef::new(r"</?[-\w]+(?:\s+[^<>]*)?\s*/?>")
            .greedy()
            .inside(tag_inside)],
    },
];

pub(crate) fn grammar() -> Grammar {
    RULES
}
