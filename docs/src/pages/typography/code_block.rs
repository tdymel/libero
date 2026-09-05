use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Text};

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

/// The demo's `source` follows its `language` - a Rust snippet under
/// `language: "python"` would show the fallback for the wrong reason - and its
/// `diff`, which needs a source written as one. Each `*_DIFF`'s `+` lines are
/// exactly its plain twin, so the diff reads as the edit that produced the
/// example. Returns the const's name alongside it, since the code block prints
/// the name a caller would write.
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
                    .doc("The text to render, highlighted when `language` names a grammar this build compiles in. Line numbers and the copy button need a real string, so this is the only way to pass content."),
                prop("language", "Language").doc(LANGUAGE_DOC),
                prop("header", "bool")
                    .default("true")
                    .doc("A bar above the code naming the language, or \"Unrecognized language\" if it isn't in the catalog or its `code-lang-*` feature is off."),
                prop("copyable", "bool")
                    .default("true")
                    .doc("Without `header`, floats in the top-right corner."),
                prop("max_lines", "Option<u32>")
                    .doc("Caps the visible height to roughly this many lines and scrolls past it; unset grows to fit. Long lines always scroll horizontally regardless."),
                prop("line_numbers", "bool")
                    .default("true")
                    .doc("Toggles the line-number gutter."),
                prop("highlight_lines", "Option<String>")
                    .doc("1-indexed lines to emphasize, e.g. `\"1,5-7,10\"`. Malformed segments are skipped, not rejected. A range past the last line stops at it."),
                prop("diff", "bool")
                    .default("false")
                    .doc("Reads `source` as a unified diff: a leading `+`/`-` colors the row and is stripped from what's shown, highlighted and copied. Wins over `highlight_lines`."),
            ])],
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "pre" }
                    "-wrapped, multi-line block with a line-number gutter, a copy button and a "
                    "header naming the language. Content is "
                    Code { source: "source" }
                    ", a plain string - line numbers and the copy button need one to split. "
                    "Add "
                    Code { source: "language" }
                    " to syntax-highlight it. For a snippet inside a sentence, use "
                    Code { source: "Code" }
                    ". Background, border, line numbers and the copy button come from "
                    Code { source: "Theme.code_block" }
                    ", the font and token colors from "
                    Code { source: "Theme.code" }
                    "; the diff and highlight tints derive from the theme's success, error and "
                    "primary colors."
                }
            },
            Demo {
                    component: "CodeBlock",
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
