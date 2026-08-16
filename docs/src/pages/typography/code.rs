use dioxus::prelude::*;
use libero::components::{Code, Flex, Text, Title};

const RUST_EXAMPLE: &str = r#"fn shout(word: &str) -> String {
    // Rust
    format!("{}!", word.to_uppercase())
}"#;

const SHELL_EXAMPLE: &str =
    "if [ -f Cargo.toml ]; then\n  cargo build --release # release build\nfi";

const MARKDOWN_EXAMPLE: &str =
    "# Libero\n\nA Dioxus component library, focused on *DX* and **a11y**.";

const HTML_EXAMPLE: &str = "<div class=\"card\">\n  <h2>Title</h2>\n</div>";

const CSS_EXAMPLE: &str = ".card {\n  color: red;\n  padding: 8px;\n}";

#[component]
pub fn CodePage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Code" }
                Text {
                    "Inline "
                    Code { "code" }
                    " by default, or a "
                    Code { "pre" }
                    "-wrapped block. Pass "
                    Code { "source" }
                    " and "
                    Code { "language" }
                    " to syntax-highlight a runtime string instead of plain "
                    Code { "children" }
                    "."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "No language" }
                Text {
                    "No "
                    Code { "language" }
                    ", or one that isn't recognized - both are the same state, so both render "
                    "the same: unhighlighted text, still with the header, line numbers, and "
                    "copy button, since those only need the source text."
                }
                Code { block: true, source: "cargo add libero" }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Highlighted - Rust" }
                Code { block: true, source: RUST_EXAMPLE, language: "rust" }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Highlighted - Shell" }
                Code { block: true, source: SHELL_EXAMPLE, language: "shell" }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Highlighted - Markdown" }
                Code { block: true, source: MARKDOWN_EXAMPLE, language: "markdown" }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Highlighted - HTML" }
                Code { block: true, source: HTML_EXAMPLE, language: "html" }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Highlighted - CSS" }
                Code { block: true, source: CSS_EXAMPLE, language: "css" }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Inline highlighting" }
                Text {
                    "Works inline too - "
                    Code { source: "let x: u32 = 5;", language: "rust" }
                    " stays a single line, colored the same way."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Without a header" }
                Text {
                    Code { "header: false" }
                    " drops the bar - the copy button (if "
                    Code { "copyable" }
                    ") floats in the top-right corner instead, aligned to the first line "
                    "either way."
                }
                Code { block: true, source: "cargo build --release", language: "shell", header: false }
                Code { block: true, source: RUST_EXAMPLE, language: "rust", header: false }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Without a copy button" }
                Text { Code { "copyable: false" } " keeps the header, drops the button." }
                Code {
                    block: true,
                    source: CSS_EXAMPLE,
                    language: "css",
                    copyable: false,
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Opaque children" }
                Text {
                    "Without "
                    Code { "source" }
                    ", "
                    Code { "children" }
                    " still works (e.g. for markup richer than plain text) - same header "
                    "chrome, but no line numbers or copy button, since there's no string to "
                    "split or copy."
                }
                Code { block: true,
                    "cargo add libero"
                }
            }
        }
    }
}
