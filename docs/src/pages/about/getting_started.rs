use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Divider, Flex, Text};

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
            title: "Getting Started",
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
                title: "Quick Start",
                Text {
                    "Wrap your app in "
                    Code { source: "LiberoProvider" }
                    " once, at the root - it registers the theme and every style your components use."
                }
                CodeBlock { source: QUICK_START_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Building for the Web",
                Text {
                    "Dioxus's "
                    Code { source: "wasm-split" }
                    " feature puts every route in its own chunk, fetched when it is first "
                    "visited instead of bloating every page's initial bundle. On this docs site "
                    "that is 297 KB of brotli-compressed main bundle instead of 414 KB. Libero "
                    "adds no split points of its own - the per-route chunks already carry "
                    Code { source: "Code" }
                    "'s highlighter and "
                    Code { source: "QrCode" }
                    "'s encoder to the pages that use them."
                }
                CodeBlock {
                    source: "dioxus = {{ version = \"*\", features = [\"router\", \"wasm-split\"] }}",
                    language: "toml",
                }
                Text {
                    "It is experimental, and "
                    Code { source: "dx" }
                    " only enables it when asked - with the feature on, always build and serve "
                    "with "
                    Code { source: "--wasm-split" }
                    ", or the app will fail to load entirely (a dangling module import, not a "
                    "graceful fallback)."
                }
                CodeBlock {
                    source: "dx serve --platform web --release --debug-symbols=false --wasm-split",
                    language: "shell",
                }
                Text {
                    "Don't need it? Drop the feature and skip "
                    Code { source: "--wasm-split" }
                    " entirely - the app renders identically either way, just from one bundle "
                    "instead of per-route chunks. Either way, keep only the languages your own "
                    "examples use:"
                }
                CodeBlock {
                    source: "libero = {{ version = \"*\", default-features = false, features = [\"code-lang-rust\"] }}",
                    language: "toml",
                }
            }

            Flex {
                direction: "column",
                gap: "lg",
                Divider {}
                Text {
                    "Styling, Theming and Performance cover how the library works; the rest "
                    "of the sidebar is one page per component."
                }
            }
        }
    }
}
