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

const LONG_RUST_EXAMPLE: &str = r#"struct Config {
    name: String,
    retries: u32,
}

impl Config {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), retries: 3 }
    }

    fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }
}"#;

const LONG_LINE_EXAMPLE: &str = r#"let url = "https://example.com/api/v2/accounts/12345/transactions?from=2024-01-01&to=2024-12-31&status=settled";"#;

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
                Title { variant: "h2", "Recognized, but not enabled" }
                Text {
                    "Libero recognizes far more languages than any one build compiles in - "
                    Code { "language: \"python\"" }
                    " is a real, known name, but this site only turns on "
                    Code { "code-lang-rust" }
                    ", "
                    Code { "code-lang-bash" }
                    ", "
                    Code { "code-lang-markdown" }
                    ", "
                    Code { "code-lang-html" }
                    ", and "
                    Code { "code-lang-css" }
                    " - so it still falls back to unhighlighted text, the same as an "
                    "unrecognized name would."
                }
                Code { block: true, source: "print(\"hello\")", language: "python" }
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
                Title { variant: "h2", "Max lines" }
                Text {
                    Code { "max_lines" }
                    " caps the visible height to roughly that many lines, scrolling "
                    "vertically past it instead of growing the block forever."
                }
                Code {
                    block: true,
                    source: LONG_RUST_EXAMPLE,
                    language: "rust",
                    max_lines: 6,
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Long lines" }
                Text { "A single line wider than the block scrolls horizontally on its own." }
                Code { block: true, source: LONG_LINE_EXAMPLE, language: "rust" }
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
