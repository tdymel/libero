use super::md;
use crate::components::form::rich_text_editor::model::{
    BlockKind, Doc, Href, Inline, Mark, MarkKind, NodeRegistry, NodeSpec,
};

/// Markdown whose model must survive `emit` then `parse` unchanged.
const CORPUS: &[&str] = &[
    "",
    "plain text",
    "two\n\nparagraphs",
    "soft\nbreak joins",
    "hard  \nbreak",
    "hard\\\nbreak",
    "# H1\n\n## H2\n\n### H3\n\n#### H4\n\n##### H5\n\n###### H6",
    "Setext\n======\n\nSub\n---",
    "*em* **strong** ***both*** ~~strike~~ `code`",
    "**bold *nested italic* bold**",
    "*a **b** c*",
    "**ab**cd *ef*gh",
    "mixed **bo*th***",
    "<u>under</u> and <u>**bold under**</u>",
    "`` a ` b ``",
    "` `` `",
    "```` ``` ````",
    "[link](https://a.example) and [titled](https://b.example \"The title\")",
    "[**bold link**](https://a.example)",
    "**[link in bold](https://a.example)**",
    "[a](https://a.example)[b](https://b.example)",
    "<https://auto.example> <mail@example.com>",
    "[spaces](<https://a.example/x y>)",
    "[parens](https://a.example/(x))",
    "> quote\n>\n> > nested",
    "> # heading in quote\n>\n> - list in quote",
    "- a\n- b\n- c",
    "* star\n+ plus",
    "1. one\n2. two",
    "7. starts at seven\n8. next",
    "1) paren",
    "- a\n  - nested\n    - deeper\n- b",
    "1. a\n\n   second paragraph\n2. b",
    "- [ ] not a task list here",
    "- \n- empty item above",
    "- a\n\n<!-- -->\n\n- b",
    "---\n\n***\n\n___",
    "```rust\nfn main() {}\n```",
    "```\nno language\n```",
    "```unknown-lang {.attr}\nkept raw\n```",
    "    indented code",
    "~~~\n```inside```\n~~~",
    "```\n\n```",
    "<div>html block</div>",
    "inline <span>html</span> stays text",
    "escapes: \\* \\_ \\` \\[ \\] \\< \\> \\# \\~ \\& \\! \\\\",
    "1\\. not a list\n\n\\- not a bullet\n\n\\+ nor this\n\n\\# not a heading\n\n\\> not a quote",
    "a\\\n\\- after break",
    "a\\\n\\=\\=\\=",
    "&amp; &lt; &copy; &#35;",
    "snake_case_word and 2*3*4",
    "trailing spaces are dropped   ",
    "unicode: 日本語 👨‍👩‍👧 é",
    "![image alt](https://img.example/a.png) keeps alt text",
    "[evil](javascript:alert(1)) stays text",
    "- **bold item**\n- `code item`",
    "> ```\n> code in quote\n> ```",
    "- ```\n  code in item\n  ```",
    "text\n\n- list right after",
    "# heading with `code` and *em*",
    "a | b | c",
];

#[test]
fn the_corpus_round_trips() {
    for source in CORPUS {
        let first = Doc::from_markdown(source);
        let written = first.to_markdown();
        let second = Doc::from_markdown(&written);
        assert_eq!(
            first.rekeyed(),
            second.rekeyed(),
            "\nsource: {source:?}\nwritten: {written:?}"
        );
        assert_eq!(
            written,
            second.to_markdown(),
            "emit is stable for {source:?}"
        );
    }
}

#[test]
fn emit_is_readable() {
    let cases = [
        ("*em* __strong__", "*em* **strong**"),
        ("Setext\n===", "# Setext"),
        ("* a\n* b", "- a\n- b"),
        ("***", "---"),
        ("soft\nbreak", "soft break"),
        ("hard  \nbreak", "hard\\\nbreak"),
        ("~~~py\nx\n~~~", "```py\nx\n```"),
        (
            "<mail@example.com>",
            "[mail@example.com](mailto:mail@example.com)",
        ),
        ("1. a\n1. b", "1. a\n2. b"),
    ];
    for (source, expected) in cases {
        assert_eq!(md(&Doc::from_markdown(source)), expected, "from {source:?}");
    }
}

#[test]
fn adjacent_lists_stay_apart() {
    let doc = Doc::from_markdown("- a\n\n* b\n\n- c");
    assert_eq!(doc.blocks.len(), 3);
    assert_eq!(md(&doc), "- a\n\n* b\n\n- c");
}

#[test]
fn unknown_fence_language_is_kept_raw() {
    let doc = Doc::from_markdown("```made-up-lang\nx\n```");
    assert_eq!(doc.blocks[0].kind, BlockKind::code("made-up-lang"));
    assert_eq!(md(&doc), "```made-up-lang\nx\n```");
}

#[test]
fn code_blocks_drop_only_the_closing_newline() {
    let doc = Doc::from_markdown("```\na\n\n```");
    assert_eq!(doc.blocks[0].text(), "a\n");
}

#[test]
fn fences_outgrow_the_backticks_inside() {
    let doc = Doc::from_markdown("````\n```\n````");
    assert_eq!(md(&doc), "````\n```\n````");
}

#[test]
fn html_never_survives_as_html() {
    let doc = Doc::from_markdown("<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>");
    for block in &doc.blocks {
        assert_eq!(block.kind, BlockKind::Paragraph);
    }
    let written = doc.to_markdown();
    assert!(!written.contains("<script>"), "{written}");
    assert!(written.contains("\\<script\\>"), "{written}");
    let bare = written
        .match_indices('<')
        .any(|(at, _)| !written[..at].ends_with('\\'));
    assert!(!bare, "an unescaped `<`: {written}");
}

#[test]
fn only_underline_tags_become_marks() {
    let doc = Doc::from_markdown("<u>x</u> <b>y</b>");
    let Inline::Text { marks, .. } = &doc.blocks[0].inlines()[0] else {
        panic!("text first")
    };
    assert!(marks.has(MarkKind::Underline));
    assert_eq!(doc.blocks[0].text(), "x <b>y</b>");
}

const XSS_HREFS: &[&str] = &[
    "javascript:alert(1)",
    "JAVASCRIPT:alert(1)",
    " javascript:alert(1)",
    "java\tscript:alert(1)",
    "java\nscript:alert(1)",
    "\u{1}javascript:alert(1)",
    "jav&#x09;ascript:alert(1)",
    "data:text/html,<script>alert(1)</script>",
    "vbscript:msgbox(1)",
    "file:///etc/passwd",
    "blob:https://a.example/x",
    "about:blank",
    "tel:+123",
    "//evil.example",
    "/relative",
    "#fragment",
    "",
    "http:",
    "https",
];

#[test]
fn unsafe_link_schemes_are_refused() {
    for href in XSS_HREFS {
        assert!(Href::parse(href).is_err(), "accepted {href:?}");
        let doc = Doc::from_markdown(&format!("[x](<{href}>)"));
        let marked = doc.blocks[0].inlines().iter().any(
            |inline| matches!(inline, Inline::Text { marks, .. } if marks.has(MarkKind::Link)),
        );
        assert!(!marked, "linked {href:?}");
    }
}

#[test]
fn safe_link_schemes_are_kept_and_normalised() {
    for (href, stored) in [
        ("https://a.example", "https://a.example"),
        ("HTTP://a.example", "http://a.example"),
        ("  https://a.example  ", "https://a.example"),
        ("mailto:me@example.com", "mailto:me@example.com"),
    ] {
        assert_eq!(Href::parse(href).map(String::from), Ok(stored.to_string()));
    }
}

#[test]
fn stored_json_cannot_smuggle_an_unsafe_link() {
    let json = r#"{"version":1,"blocks":[{"key":1,"type":"paragraph","inlines":[
        {"type":"text","text":"x","marks":[{"type":"link","href":"javascript:alert(1)"}]}]}]}"#;
    assert!(serde_json::from_str::<Doc>(json).is_err());
}

#[test]
fn custom_nodes_write_through_the_registry() {
    let mut registry = NodeRegistry::default();
    registry
        .register(NodeSpec::inline("mention").markdown(|attrs, _| {
            format!(
                "@{}",
                attrs.get("user").and_then(|v| v.as_str()).unwrap_or("")
            )
        }))
        .unwrap();
    let mut doc = Doc::from_markdown("hi");
    doc.blocks[0].inlines_mut().push(Inline::text(" "));
    let mention = registry
        .new_inline("mention", [("user".to_string(), "ann".into())].into())
        .unwrap();
    doc.blocks[0].inlines_mut().push(mention);
    assert_eq!(doc.to_markdown_with(&registry), "hi @ann\n");
    assert_eq!(
        doc.to_markdown(),
        "hi \n",
        "no codec: the node writes nothing"
    );
}

/// Random mark combinations over words: what the editor produces, not what Markdown parses to.
#[test]
fn random_marked_runs_round_trip() {
    let words = [
        "ab", "x", "日本", "a1", "(y)", "z.", "\"q\"", "-", "_u_", "*s*", "`t`",
    ];
    let link = Mark::link("https://a.example/p?q=1").unwrap();
    let other_link = Mark::link("mailto:me@example.com").unwrap();
    let pool = [
        Mark::Bold,
        Mark::Italic,
        Mark::Underline,
        Mark::Strike,
        Mark::Code,
        link,
        other_link,
    ];
    let mut seed: u64 = 0x2545_F491_4F6C_DD1D;
    let mut next = |below: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % below as u64) as usize
    };
    let mut failures = Vec::new();
    for _ in 0..500 {
        let mut inlines = Vec::new();
        for index in 0..1 + next(6) {
            if index > 0 && next(2) == 0 {
                inlines.push(Inline::text(" "));
            }
            let mut marks = crate::components::form::rich_text_editor::model::Marks::new();
            for mark in &pool {
                if next(3) == 0 {
                    marks.add(mark.clone());
                }
            }
            inlines.push(Inline::marked(words[next(words.len())], marks));
        }
        let mut doc = Doc::from_markdown("x");
        *doc.blocks[0].inlines_mut() = inlines;
        doc.normalize();
        let written = doc.to_markdown();
        let back = Doc::from_markdown(&written);
        if back.blocks[0].inlines() != doc.blocks[0].inlines() {
            failures.push(written);
        }
    }
    assert!(
        failures.is_empty(),
        "{} of 500 failed, e.g. {:#?}",
        failures.len(),
        &failures[..failures.len().min(8)]
    );
}

#[test]
fn marks_split_by_a_link_stay_nested() {
    let mut doc = Doc::from_markdown("x");
    let link = Mark::link("https://a.example").unwrap();
    *doc.blocks[0].inlines_mut() = vec![
        Inline::marked("a", [Mark::Bold].into_iter().collect()),
        Inline::marked("b", [Mark::Bold, link.clone()].into_iter().collect()),
        Inline::marked("c", [link].into_iter().collect()),
    ];
    let written = doc.to_markdown();
    let back = Doc::from_markdown(&written);
    assert_eq!(
        back.blocks[0].inlines(),
        doc.blocks[0].inlines(),
        "{written}"
    );
}
