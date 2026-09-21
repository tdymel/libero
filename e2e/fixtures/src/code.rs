//! `Code` and `CodeBlock`, on the browser's regex engine.

use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/code", || rsx! { CodePage {} })];

/// Tests the highlighter's wasm `RegExp` arm, which `cargo test` never runs
/// (`codebase/highlighter-regex-engines`, todo 279).
#[component]
fn CodePage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Code { id: "non-ascii-code", language: "rust", source: NON_ASCII_SOURCE }
            CodeBlock { id: "numbered-block", language: "rust", line_numbers: true, source: "let a = 1;\nlet b = 2;" }
            // Todo 434: the leading `é` puts the scan's byte offsets apart from `RegExp`'s UTF-16 ones.
            Code { id: "nested-comment-code", language: "rust", source: "é /* a /* b */ c */ x" }
            CodeBlock { id: "diff-block", language: "rust", header: true, copyable: true, diff: true,
                source: "fn greet() {{\n-    old();\n+    new();\n}}"
            }
            // Todo 668: no grammar, still marked.
            CodeBlock { id: "plain-diff-block", diff: true, highlight_lines: "1",
                header: false, copyable: false, line_numbers: false,
                source: "keep\n-old\n+new"
            }
            CodeBlock { id: "wide-block", language: "rust", copyable: true, highlight_lines: "2",
                // One token that starts in view: axe skips a token scrolled out whole.
                source: "fn main() {{\n    let s = \"a string long enough to scroll the block sideways\";\n}}"
            }
            // Todo 771: no gutter digit holds the blank line's height.
            CodeBlock { id: "blank-line-block", language: "rust", line_numbers: false, copyable: false,
                source: "let a = 1;\n\nlet b = 2;"
            }
            // Todo 732: a shrink-to-fit parent, where `anywhere` split a short span after its `#`.
            div { style: "width: min-content",
                Text { "Add " Code { id: "short-code", source: "#[derive(Options)]" } " above it." }
            }
            Text { id: "long-code-text",
                Code { id: "long-code", source: "an_identifier_far_too_long_for_a_three_hundred_twenty_pixel_column" }
            }
        }
    }
}

/// `\b` is ASCII in JS, Unicode in `regex`: the `if` in `éif` differed. The second `if` is the control.
/// Keep in step with libero's `a_keyword_after_a_non_ascii_letter_tokenizes_as_it_does_on_the_web`.
const NON_ASCII_SOURCE: &str = "éif x; if y";
