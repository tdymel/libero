use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, CodeBlockPart, Text};

use super::code::LANGUAGE_DOC;

const RUST_EXAMPLE: &str = r#"fn shout(word: &str) -> String {
    // Rust
    format!("{}!", word.to_uppercase())
}"#;

// snippet: ignore - a diff
const RUST_DIFF: &str = r#"fn shout(word: &str) -> String {
    // Rust
-    format!("{}", word)
+    format!("{}!", word.to_uppercase())
}"#;

/// A real, recognized language this build doesn't compile in - the demo's
/// "python" option shows that falling back to plain text.
const PYTHON_EXAMPLE: &str = "def shout(word):\n    # Python\n    return word.upper() + \"!\"";

const PYTHON_DIFF: &str =
    "def shout(word):\n    # Python\n-    return word.upper()\n+    return word.upper() + \"!\"";

/// The source for `language` and `diff`, with the const's name the code block prints.
/// Each `*_DIFF`'s `+` lines are exactly its plain twin.
fn example(values: &DemoValues) -> (&'static str, &'static str) {
    match (
        values.str("language").as_str(),
        values.str("diff") == "true",
    ) {
        ("python", false) => ("PYTHON_EXAMPLE", PYTHON_EXAMPLE),
        ("python", true) => ("PYTHON_DIFF", PYTHON_DIFF),
        (_, false) => ("RUST_EXAMPLE", RUST_EXAMPLE),
        (_, true) => ("RUST_DIFF", RUST_DIFF),
    }
}

#[component]
pub fn CodeBlockPage() -> Element {
    rsx! {
        DocPage {
            title: "CodeBlock",
            source: "libero/src/components/typography/code/code_block.rs",
            markdown: "/md/code_block.md",
            properties: vec![props("CodeBlock", vec![
                prop("source", "String")
                    .default("required")
                    .doc("The text. Highlighted when `language` names a grammar this build compiles in."),
                prop("language", "Language").doc(LANGUAGE_DOC),
                prop("header", "bool")
                    .default("true")
                    .doc("A bar above the code naming the language. Left out when there is no language and no copy button."),
                prop("copyable", "bool")
                    .default("true")
                    .doc("Shows a `Copy`. Without `header`, it floats in the top-right corner."),
                prop("max_lines", "u32")
                    .doc("Caps the height at about this many lines and scrolls the rest. Unset, the block grows to fit."),
                prop("line_numbers", "bool")
                    .default("true")
                    .doc("Shows the line-number gutter."),
                prop("highlight_lines", "String")
                    .doc("Lines to emphasize, counted from 1, such as `\"1,5-7,10\"`. Malformed parts are skipped."),
                prop("diff", "bool")
                    .default("false")
                    .doc("Reads `source` as a unified diff. A leading `+` or `-` colors the row and stays out of what is copied. Wins over `highlight_lines`."),
                prop("label", "String")
                    .doc("Names the block and describes its copy button, such as \"The booking card, Rust code\". Unset, the language, such as \"Rust code\"."),
                prop("parts", "Parts<CodeBlockPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("CodeBlockPart", vec![
                (CodeBlockPart::Header, "The bar above the code, with `header`."),
                (CodeBlockPart::Language, "The language name in the header."),
                (CodeBlockPart::Copy, "The copy button, in the header or floating in the corner."),
                (CodeBlockPart::Scroll, "The scrolling box round the code."),
            ])],
            accessibility: a11y()
                .handles([
                    "The block is a `group` named by `label`. Its copy button is \"Copy code\", described by the same words, so a screen reader hears \"Copy code, Rust code\".",
                    "In `diff` mode a screen reader hears \"added\" or \"removed\" before a changed line.",
                    "A block that scrolls is a focusable region named after its language, such as \"Rust code\". The words come from the localization.",
                ])
                .must([
                    "With several blocks on a page, give each a `label` that says what the code is, such as \"The booking form, Rust code\".",
                    "A line in `highlight_lines` is marked only by color and a bar, so say in the text why it matters.",
                ]),
            lead: rsx! {
                Text {
                    "A multi-line code block with line numbers, a copy button and a header "
                    "naming the language. Pass the text as "
                    Code { source: "source" }
                    " and set "
                    Code { source: "language" }
                    " to highlight it. For a snippet inside a sentence, use "
                    Code { source: "Code" }
                    "."
                }
            },
            // snippet: item const RUST_EXAMPLE: &str = "";
            // snippet: item const PYTHON_EXAMPLE: &str = "";
            // snippet: item const RUST_DIFF: &str = "";
            // snippet: item const PYTHON_DIFF: &str = "";
            Demo {
                    component: "CodeBlock",
                    children_text: "",
                    controls: vec![
                        // Picks `source` as well, so it always prints. "none" is the real default.
                        Control::toggle("language", ["rust", "python"])
                            // `python` is a real, recognized name whose
                            // `code-lang-*` feature this build leaves off.
                            .labels(["Rust", "Python (off)"])
                            .code(|_, values| {
                                vec![
                                    format!("source: {}", example(values).0),
                                    format!("language: {:?}", values.str("language")),
                                ]
                            }),
                        Control::switch("diff"),
                        Control::switch("header").default("true"),
                        Control::switch("copyable").default("true"),
                        Control::switch("line_numbers").default("true"),
                        Control::switch("highlight_lines").code(|_, values| {
                            match values.str("highlight_lines") == "true" {
                                true => vec!["highlight_lines: \"2,3\"".to_string()],
                                false => vec![],
                            }
                        }),
                        Control::slider("max_lines", ["auto", "2", "3"]).code(|_, values| {
                            match values.str("max_lines").as_str() {
                                "auto" => vec![],
                                lines => vec![format!("max_lines: {lines}")],
                            }
                        }),
                    ],
                    render: move |values: DemoValues| {
                        rsx! {
                            CodeBlock {
                                source: example(&values).1,
                                language: values.str("language"),
                                diff: values.str("diff") == "true",
                                header: values.str("header") == "true",
                                copyable: values.str("copyable") == "true",
                                line_numbers: values.str("line_numbers") == "true",
                                highlight_lines: match values.str("highlight_lines").as_str() {
                                    "true" => Some("2,3".to_string()),
                                    _ => None,
                                },
                                max_lines: values.str("max_lines").parse::<u32>().ok(),
                            }
                        }
                    },
            }
        }
    }
}
