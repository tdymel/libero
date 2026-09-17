use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Code, CodeBlock, Table, Text, column},
    sx::sx,
};

const FEATURES: [(&str, &str); 3] = [
    (
        "code-lang-<name>",
        "One grammar for Code and CodeBlock, 30 in all. Rust, Bash, Markdown, HTML and CSS are the default.",
    ),
    (
        "full-polymorphism",
        "Box renders the rarer HTML elements too (metadata, media, web components). Without it they fall back to a div.",
    ),
    (
        "native",
        "Element access through Blitz, for apps on dioxus-native.",
    ),
];

// snippet: ignore - a Cargo.toml fragment, not Rust
const FEATURES_EXAMPLE: &str = r#"libero = { version = "0.1", default-features = false, features = [
    "code-lang-rust",
    "full-polymorphism",
] }"#;

const QUICK_START_EXAMPLE: &str = r#"fn App() -> Element {
    rsx! {
        LiberoProvider {
            Text { "Hello, Libero!" }
        }
    }
}"#;

#[component]
pub fn GettingStarted() -> Element {
    rsx! {
        DocPage {
            title: "Getting started",
            markdown: "/md/getting_started.md",
            lead: rsx! {
                Text {
                    "Libero is a Dioxus component library focused on developer experience, UX, accessibility, and configurability."
                }
            },
            DocSection {
                title: "Installation",
                Text { "Add Libero to your project with cargo:" }
                CodeBlock { source: "cargo add libero", language: "shell" }
            }

            DocSection {
                title: "Quick start",
                Text {
                    "Wrap your app in "
                    Code { source: "LiberoProvider" }
                    " once, at the root. It registers the theme and every style your components use."
                }
                CodeBlock { source: QUICK_START_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Building for the web",
                Text {
                    "Dioxus's "
                    Code { source: "wasm-split" }
                    " feature puts every route in its own chunk, fetched on its first visit, "
                    "so the initial bundle stays small. On this docs site the brotli-compressed "
                    "main bundle is 297 KB instead of 414 KB. Libero adds no split points of its "
                    "own. The per-route chunks already carry "
                    Code { source: "Code" }
                    "'s highlighter and "
                    Code { source: "QrCode" }
                    "'s encoder to the pages that use them."
                }
                CodeBlock {
                    source: "dioxus = {{ version = \"0.8.0-alpha.1\", features = [\"router\", \"wasm-split\"] }}",
                    language: "toml",
                }
                Text {
                    "It is experimental, and "
                    Code { source: "dx" }
                    " enables it only when asked. With the feature on, always build and serve "
                    "with "
                    Code { source: "--wasm-split" }
                    ", or the app fails to load."
                }
                CodeBlock {
                    source: "dx serve --platform web --release --debug-symbols=false --wasm-split",
                    language: "shell",
                }
                Text {
                    "Without it, drop the feature and the "
                    Code { source: "--wasm-split" }
                    " flag. The app renders the same, from one bundle. Either way, keep only "
                    "the languages your own examples use:"
                }
                CodeBlock {
                    source: "libero = {{ version = \"0.1\", default-features = false, features = [\"code-lang-rust\"] }}",
                    language: "toml",
                }
            }

            DocSection {
                title: "Feature flags",
                Text { "Every feature is additive." }
                Table {
                    aria_label: "Feature flags",
                    data: FEATURES.to_vec(),
                    columns: vec![
                        column("Flag")
                            .value(|row: &(&str, &str)| row.0)
                            .render(|row: &(&str, &str)| rsx! { Code { source: row.0, sx: sx().white_space("nowrap") } }),
                        column("What it does").value(|row: &(&str, &str)| row.1),
                    ],
                }
                CodeBlock { source: FEATURES_EXAMPLE, language: "toml" }
            }
        }
    }
}
