use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Code, Divider, Flex, Text},
    sx::sx,
};

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
            lead: rsx! {
                Text {
                    "Libero is a Dioxus component library focused on developer experience, UX, accessibility, and configurability."
                }
            },
            DocSection {
                title: "Installation",
                Text { "Add Libero to your project with cargo:" }
                Code { block: true, source: "cargo add libero", language: "shell" }
            }

            DocSection {
                title: "Quick Start",
                Text {
                    "Wrap your app in "
                    Code { "LiberoProvider" }
                    " once, at the root - it registers the theme and every style your components use."
                }
                Code { block: true, source: QUICK_START_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Building for the Web",
                Text {
                    "Dioxus's "
                    Code { "wasm-split" }
                    " feature puts every route in its own chunk, fetched when it is first "
                    "visited instead of bloating every page's initial bundle. On this docs site "
                    "that is 297 KB of brotli-compressed main bundle instead of 414 KB. Libero "
                    "adds no split points of its own - the per-route chunks already carry "
                    Code { "Code" }
                    "'s highlighter and "
                    Code { "QrCode" }
                    "'s encoder to the pages that use them."
                }
                Code {
                    block: true,
                    source: "dioxus = {{ version = \"*\", features = [\"router\", \"wasm-split\"] }}",
                    language: "toml",
                }
                Text {
                    "It is experimental, and "
                    Code { "dx" }
                    " only enables it when asked - with the feature on, always build and serve "
                    "with "
                    Code { "--wasm-split" }
                    ", or the app will fail to load entirely (a dangling module import, not a "
                    "graceful fallback)."
                }
                Code {
                    block: true,
                    source: "dx serve --platform web --release --debug-symbols=false --wasm-split",
                    language: "shell",
                }
                Text {
                    "Don't need it? Drop the feature and skip "
                    Code { "--wasm-split" }
                    " entirely - the app renders identically either way, just from one bundle "
                    "instead of per-route chunks. Either way, keep only the languages your own "
                    "examples use:"
                }
                Code {
                    block: true,
                    source: "libero = {{ version = \"*\", default-features = false, features = [\"code-lang-rust\"] }}",
                    language: "toml",
                }
            }

            Flex {
                direction: "column",
                gap: "lg",
                Divider {}
                Text {
                    sx: sx().color("grey.6"),
                    "More documentation is on the way."
                }
            }
        }
    }
}
