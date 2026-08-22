use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Text};

use super::code::languages_section;

const RUST_EXAMPLE: &str = r#"fn shout(word: &str) -> String {
    // Rust
    format!("{}!", word.to_uppercase())
}"#;

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
            DocSection {
                title: "Usage",
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
            {languages_section()}
        }
    }
}
