use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Divider, Flex, Text};

// snippet: ignore - a Cargo.toml fragment, not Rust
const FEATURES_EXAMPLE: &str = r#"libero = { version = "*", default-features = false, features = [
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

            DocSection {
                title: "Feature flags",
                Text {
                    "Every libero feature is additive, and the default set is five "
                    Code { source: "code-lang-*" }
                    " grammars: Rust, Bash, Markdown, HTML and CSS. Two are worth knowing about."
                }
                Text {
                    Code { source: "code-lang-*" }
                    " compiles one hand-ported grammar each for "
                    Code { source: "Code" }
                    " and "
                    Code { source: "CodeBlock" }
                    ". There are 30 of them; with "
                    Code { source: "default-features = false" }
                    " you pay only for the ones you name."
                }
                Text {
                    Code { source: "full-polymorphism" }
                    " widens what "
                    Code { source: "Box" }
                    "'s "
                    Code { source: "component" }
                    " prop can render. All 111 HTML5 element names are accepted and type-check "
                    "either way, but only 83 of them compile a match arm by default: every "
                    "sectioning, text-level, list, table and form element - "
                    Code { source: "footer" }
                    ", "
                    Code { source: "strong" }
                    ", "
                    Code { source: "em" }
                    ", "
                    Code { source: "small" }
                    ", "
                    Code { source: "time" }
                    ", "
                    Code { source: "details" }
                    ", "
                    Code { source: "dialog" }
                    " and the rest. The feature adds the remaining 28, which are document "
                    "metadata ("
                    Code { source: "head" }
                    ", "
                    Code { source: "meta" }
                    ", "
                    Code { source: "title" }
                    ", "
                    Code { source: "script" }
                    ", "
                    Code { source: "style" }
                    ", "
                    Code { source: "link" }
                    ", "
                    Code { source: "base" }
                    ", "
                    Code { source: "body" }
                    ", "
                    Code { source: "noscript" }
                    "), embedded and media content ("
                    Code { source: "iframe" }
                    ", "
                    Code { source: "canvas" }
                    ", "
                    Code { source: "audio" }
                    ", "
                    Code { source: "video" }
                    ", "
                    Code { source: "picture" }
                    ", "
                    Code { source: "source" }
                    ", "
                    Code { source: "track" }
                    ", "
                    Code { source: "embed" }
                    ", "
                    Code { source: "object" }
                    ", "
                    Code { source: "param" }
                    ", "
                    Code { source: "map" }
                    ", "
                    Code { source: "area" }
                    "), web components ("
                    Code { source: "template" }
                    ", "
                    Code { source: "slot" }
                    ") and the bidi and ruby set ("
                    Code { source: "bdi" }
                    ", "
                    Code { source: "bdo" }
                    ", "
                    Code { source: "ruby" }
                    ", "
                    Code { source: "rp" }
                    ", "
                    Code { source: "rt" }
                    ")."
                }
                Text {
                    "Pass one of those 28 without the feature and you get a "
                    Code { source: "div" }
                    " - with a console warning in a debug build, and silently in a release "
                    "build, because the warning compiles to nothing there. On this docs site "
                    "the feature costs 14.8 KB of wasm, 1.0 KB after brotli."
                }
                CodeBlock {
                    source: FEATURES_EXAMPLE,
                    language: "toml",
                }
                Text {
                    Code { source: "native" }
                    " is the third: it reaches elements through Blitz when you run under "
                    Code { source: "dioxus-native" }
                    ", instead of Dioxus's portable mounted handle, which cannot query a "
                    "subtree or report focus."
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
