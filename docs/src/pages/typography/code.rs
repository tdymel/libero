use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap};
use dioxus::prelude::*;
use libero::components::{Code, Text};

const RUST_EXAMPLE: &str = r#"fn shout(word: &str) -> String {
    // Rust
    format!("{}!", word.to_uppercase())
}"#;

const RUST_DIFF: &str = r#"fn shout(word: &str) -> String {
    // Rust
-    format!("{}", word)
+    format!("{}!", word.to_uppercase())
}"#;

const BASH_EXAMPLE: &str =
    "if [ -f Cargo.toml ]; then\n  cargo build --release # release build\nfi";

const BASH_DIFF: &str =
    "if [ -f Cargo.toml ]; then\n-  cargo build\n+  cargo build --release # release build\nfi";

const MARKDOWN_EXAMPLE: &str =
    "# Libero\n\nA Dioxus component library, focused on *DX* and **a11y**.";

const MARKDOWN_DIFF: &str = "# Libero\n\n-A Dioxus component library, focused on *DX*.\n+A Dioxus component library, focused on *DX* and **a11y**.";

const HTML_EXAMPLE: &str = "<div class=\"card\">\n  <h2>Title</h2>\n</div>";

const HTML_DIFF: &str = "-<div>\n+<div class=\"card\">\n  <h2>Title</h2>\n</div>";

const CSS_EXAMPLE: &str = ".card {\n  color: red;\n  padding: 8px;\n}";

const CSS_DIFF: &str = ".card {\n-  color: crimson;\n+  color: red;\n  padding: 8px;\n}";

/// A real, recognized language that this build doesn't compile in - the
/// demo's "python" option shows that falling back to plain text.
const PYTHON_EXAMPLE: &str = "def shout(word):\n    # Python\n    return word.upper() + \"!\"";

const PYTHON_DIFF: &str =
    "def shout(word):\n    # Python\n-    return word.upper()\n+    return word.upper() + \"!\"";

/// The demo's `source` follows its `language` - a Rust snippet under
/// `language: "css"` would only show the highlighter failing - and its `diff`,
/// which needs a source written as one. Every `*_DIFF`'s `+` lines are exactly
/// its plain twin, so each diff reads as the edit that produced the example.
/// Returns the const's name alongside it, since the code block prints the name
/// a caller would write.
fn example(values: &DemoValues) -> (&'static str, &'static str) {
    let diff = values.str("diff") == "true";
    match (values.str("language").as_str(), diff) {
        ("bash", false) => ("BASH_EXAMPLE", BASH_EXAMPLE),
        ("bash", true) => ("BASH_DIFF", BASH_DIFF),
        ("markdown", false) => ("MARKDOWN_EXAMPLE", MARKDOWN_EXAMPLE),
        ("markdown", true) => ("MARKDOWN_DIFF", MARKDOWN_DIFF),
        ("html", false) => ("HTML_EXAMPLE", HTML_EXAMPLE),
        ("html", true) => ("HTML_DIFF", HTML_DIFF),
        ("css", false) => ("CSS_EXAMPLE", CSS_EXAMPLE),
        ("css", true) => ("CSS_DIFF", CSS_DIFF),
        ("python", false) => ("PYTHON_EXAMPLE", PYTHON_EXAMPLE),
        ("python", true) => ("PYTHON_DIFF", PYTHON_DIFF),
        (_, false) => ("RUST_EXAMPLE", RUST_EXAMPLE),
        (_, true) => ("RUST_DIFF", RUST_DIFF),
    }
}

/// Inline `Code` earns its keep mid-sentence, so the demo shows it there
/// rather than alone: (before, source, after).
fn inline_example(language: &str) -> (&'static str, &'static str, &'static str) {
    match language {
        "bash" => (
            "Run ",
            "cargo build --release",
            " before you profile anything.",
        ),
        "markdown" => (
            "Wrap a word in ",
            "**bold**",
            " when it has to carry the sentence.",
        ),
        "html" => (
            "Every card opens with an ",
            "<h2>",
            " so the outline reads right.",
        ),
        "css" => (
            "Set ",
            "color: crimson;",
            " on the card and the heading follows.",
        ),
        "python" => ("Call ", "word.upper()", " when the label has to shout."),
        _ => (
            "Bind it with ",
            "let width: u32 = 320;",
            " before the first draw.",
        ),
    }
}

/// The inline preview draws a sentence around the `Code`, so the code block
/// has to print that sentence too.
fn wrap_inline(values: &DemoValues, code: &str) -> String {
    if values.str("block") == "true" {
        return code.to_string();
    }
    let (before, _, after) = inline_example(&values.str("language"));
    let indented: String = code.lines().map(|line| format!("    {line}\n")).collect();
    format!("Text {{\n    {before:?}\n{indented}    {after:?}\n}}")
}

/// `block`-only props print nothing while the demo is inline - `Code` ignores
/// them there, so printing them would be a lie.
fn block_only(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    match values.str("block") == "true" && value != control.default {
        true => vec![format!("{}: {value}", control.name)],
        false => vec![],
    }
}

#[component]
pub fn CodePage() -> Element {
    rsx! {
        DocPage {
            title: "Code",
            lead: rsx! {
                Text {
                    "Inline "
                    Code { source: "code" }
                    " by default, or a "
                    Code { source: "pre" }
                    "-wrapped block. Content is always "
                    Code { source: "source" }
                    ", a plain string - line numbers and the copy button need one to split. "
                    "Add "
                    Code { source: "language" }
                    " to syntax-highlight it. Every color here - background, border, line numbers, diff/highlight "
                    "tints, and the syntax token colors - comes from "
                    Code { source: "Theme.code" }
                    " and can be overridden per-app."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Code",
                    children_text: "",
                    controls: vec![
                        // Picks `source` as well, so it always prints - the
                        // preview would otherwise show code the block below
                        // never mentions. "none" is the real default.
                        Control::select(
                            "language",
                            ["rust", "bash", "markdown", "html", "css", "python"],
                        )
                            // `python` is a real, recognized name whose
                            // `code-lang-*` feature this build leaves off.
                            .labels([
                                "rust",
                                "bash",
                                "markdown",
                                "html",
                                "css",
                                "python (not enabled)",
                            ])
                            .code(|_, values| {
                                let language = values.str("language");
                                // Inline sources are one short line, so they
                                // print as a literal rather than a const.
                                let source = match values.str("block") == "true" {
                                    true => format!("source: {}", example(values).0),
                                    false => {
                                        format!("source: {:?}", inline_example(&language).1)
                                    }
                                };
                                vec![source, format!("language: {language:?}")]
                            }),
                        // On by default here, though `Code`'s own default is
                        // inline - a block is what the other props are about.
                        // The printed rsx still follows the real default.
                        Control::switch("block").default("true").code(|_, values| {
                            match values.str("block").as_str() {
                                "true" => vec!["block: true".to_string()],
                                _ => vec![],
                            }
                        }),
                        Control::switch("diff").code(block_only),
                        Control::switch("header").default("true").code(block_only),
                        Control::switch("copyable").default("true").code(block_only),
                        Control::switch("line_numbers").default("true").code(block_only),
                        Control::switch("highlight_lines").code(|_, values| {
                            match values.str("block") == "true"
                                && values.str("highlight_lines") == "true"
                            {
                                true => vec!["highlight_lines: \"2,3\"".to_string()],
                                false => vec![],
                            }
                        }),
                        Control::slider("max_lines", ["auto", "2", "3"]).code(|_, values| {
                            match values.str("max_lines").as_str() {
                                "auto" => vec![],
                                _ if values.str("block") != "true" => vec![],
                                lines => vec![format!("max_lines: {lines}")],
                            }
                        }),
                    ],
                    render: move |values: DemoValues| {
                        let language = values.str("language");
                        if values.str("block") != "true" {
                            let (before, source, after) = inline_example(&language);
                            return rsx! {
                                Text {
                                    {before}
                                    Code { source, language }
                                    {after}
                                }
                            };
                        }
                        rsx! {
                            Code {
                                block: true,
                                source: example(&values).1,
                                language,
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
                    wrap: Wrap(wrap_inline),
                }
            }
            DocSection {
                title: "Recognized, but not enabled",
                Text {
                    "Libero recognizes far more languages than any one build compiles in - "
                    Code { source: "language: \"python\"" }
                    " is a real, known name, but this site only turns on "
                    Code { source: "code-lang-rust" }
                    ", "
                    Code { source: "code-lang-bash" }
                    ", "
                    Code { source: "code-lang-markdown" }
                    ", "
                    Code { source: "code-lang-html" }
                    ", and "
                    Code { source: "code-lang-css" }
                    " - so it falls back to unhighlighted text, exactly as an unrecognized "
                    "name would. Pick "
                    Code { source: "python" }
                    " above to see it: a real language, no grammar compiled in to apply. "
                    "Leaving "
                    Code { source: "language" }
                    " off entirely lands in the same state."
                }
            }
        }
    }
}
