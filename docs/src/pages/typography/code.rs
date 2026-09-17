use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Text};

/// The `language` prop's doc, shared with the `CodeBlock` page: the prop and
/// the feature flags behind it work the same for both. Each grammar is also a
/// `language` value, though `language` takes aliases the feature names don't -
/// `rs`, `py`, `c#`.
pub(super) const LANGUAGE_DOC: &str = "One of 30 grammars, each behind its own `code-lang-*` \
feature, so a build pays only for what it highlights: bash, c, cpp, csharp, css, dart, go, graphql, \
haskell, html, java, javascript, json, kotlin, lua, markdown, objective-c, perl, php, powershell, \
python, r, ruby, rust, scala, sql, swift, toml, typescript, yaml. Aliases such as `rs`, `py` and `c#` \
work too. The default features cover `rust`, `bash`, `css` and a few more. An unknown name, or one \
whose feature is off, renders plain text.";

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
            source: "libero/src/components/typography/code/code.rs",
            markdown: "/md/code.md",
            properties: vec![props("Code", vec![
                prop("source", "String")
                    .default("required")
                    .doc("The text. Highlighted when `language` names a grammar this build compiles in."),
                prop("language", "Language").doc(LANGUAGE_DOC),
            ])],
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "<code>" }
                    " element for a snippet inside a sentence. Pass the text as "
                    Code { source: "source" }
                    " and set "
                    Code { source: "language" }
                    " to highlight it. For a multi-line block with line numbers and a copy "
                    "button, use "
                    Code { source: "CodeBlock" }
                    ". The font and the token colors come from the theme."
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
        }
    }
}
