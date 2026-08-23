use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Text};

/// Every grammar libero ships, spelled as its `code-lang-*` suffix. Each is
/// also a `language` value, though `language` takes aliases the feature names
/// don't - `rs`, `py`, `c#`.
const ALL_LANGUAGES: &str = "bash, c, cpp, csharp, css, dart, go, graphql, haskell, html, java, \
javascript, json, kotlin, lua, markdown, objective-c, perl, php, powershell, python, r, ruby, \
rust, scala, sql, swift, toml, typescript, yaml";

/// Inline `Code` earns its keep mid-sentence, so the demo shows it there
/// rather than alone: (before, source, after).
fn inline_example(language: &str) -> (&'static str, &'static str, &'static str) {
    match language {
        // A real, recognized name whose `code-lang-*` feature this build
        // leaves off, so it renders unhighlighted.
        "python" => ("Call ", "word.upper()", " when the label has to shout."),
        _ => (
            "Bind it with ",
            "let width: u32 = 320;",
            " before the first draw.",
        ),
    }
}

/// Shared with the `CodeBlock` page: `language` and the feature flags behind
/// it work the same for both.
pub(super) fn languages_section() -> Element {
    rsx! {
        DocSection {
            title: "Languages",
            Text {
                "Libero ships grammars for 30 languages, each behind its own "
                Code { source: "code-lang-*" }
                " feature so a build only pays for what it highlights. The default set covers "
                Code { source: "rust" }
                ", "
                Code { source: "bash" }
                ", "
                Code { source: "css" }
                " and a few more; enable the rest as you need them. An unrecognized name - or a "
                "recognized one whose feature is off - falls back to plain, unhighlighted text, "
                "which is also what leaving "
                Code { source: "language" }
                " off does. This site doesn't compile in "
                Code { source: "python" }
                ", so picking it above shows exactly that."
            }
            Code { source: ALL_LANGUAGES }
            Text {
                "The grammars are hand-ported from "
                Anchor { to: "https://prismjs.com", "Prism" }
                ", as is the tokenizer that runs them. These 30 are where we started, not a "
                "closed set - if you need one Prism has and we don't, it can be ported the same "
                "way."
            }
        }
    }
}

/// The preview draws a sentence around the `Code`, so the code block has to
/// print that sentence too.
fn wrap_inline(values: &DemoValues, code: &str) -> String {
    let (before, _, after) = inline_example(&values.str("language"));
    let indented = indent(code);
    format!("Text {{\n    {before:?}\n{indented}    {after:?}\n}}")
}

#[component]
pub fn CodePage() -> Element {
    rsx! {
        DocPage {
            title: "Code",
            properties: vec![props("Code", vec![
                prop("source", "String").doc("The text to render, highlighted when `language` names a grammar this build compiles in."),
                prop("language", "Language")
                    .doc("Unrecognized values fall back to no highlighting rather than a guess."),
            ])],
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "code" }
                    " element for a snippet inside a sentence. Content is "
                    Code { source: "source" }
                    ", a plain string; add "
                    Code { source: "language" }
                    " to syntax-highlight it. For a multi-line, "
                    Code { source: "pre" }
                    "-wrapped block with line numbers, a copy button and diffs, reach for "
                    Code { source: "CodeBlock" }
                    " instead. The font and the syntax token colors come from "
                    Code { source: "Theme.code" }
                    " and can be overridden per-app."
                }
            },
            Demo {
                component: "Code",
                children_text: "",
                controls: vec![
                    // Picks `source` as well, so it always prints - the
                    // preview would otherwise show code the block below
                    // never mentions. "none" is the real default.
                    Control::toggle("language", ["rust", "python"])
                        // `python` is a real, recognized name whose
                        // `code-lang-*` feature this build leaves off.
                        .labels(["rust", "python (off)"])
                        .code(|_, values| {
                            let language = values.str("language");
                            // Inline sources are one short line, so they
                            // print as a literal rather than a const.
                            vec![
                                format!("source: {:?}", inline_example(&language).1),
                                format!("language: {language:?}"),
                            ]
                        }),
                ],
                render: move |values: DemoValues| {
                    let language = values.str("language");
                    let (before, source, after) = inline_example(&language);
                    rsx! {
                        Text {
                            {before}
                            Code { source, language }
                            {after}
                        }
                    }
                },
                wrap: Wrap(wrap_inline),
            }
            {languages_section()}
        }
    }
}
