use crate::components::{DocPage, DocSection};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::{
    components::{
        Alert, Code, CodeBlock, Flex, Icon, List, ListItem, Options, Table, Tabs, Text, column,
    },
    sx::sx,
};

const FEATURES: [(&str, &str); 8] = [
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
    (
        "desktop",
        "System notifications over D-Bus in a desktop WebView on Linux, where WebKitGTK denies them.",
    ),
    (
        "icons-bootstrap",
        "The Bootstrap Icons set for IconProvider, as IconSet::bootstrap(). Without it libero's glyphs are Lucide.",
    ),
    (
        "icons-material",
        "The Material Design Icons set for IconProvider, as IconSet::material(). Without it libero's glyphs are Lucide.",
    ),
    (
        "icons-phosphor",
        "The Phosphor set for IconProvider, as IconSet::phosphor(). Without it libero's glyphs are Lucide.",
    ),
    (
        "icons-tabler",
        "The Tabler Icons set for IconProvider, as IconSet::tabler(). Without it libero's glyphs are Lucide.",
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

// snippet: ignore - shell commands, not Rust
const WEB_COMMANDS: &str = "dx serve --platform web
dx build --platform web --release";

// snippet: ignore - a Cargo.toml fragment, not Rust
const WEB_PROFILE: &str = r#"[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true"#;

// snippet: ignore - a Dioxus.toml fragment, not Rust
const WEB_DIOXUS: &str = r#"[web]
pre_compress = true"#;

// snippet: ignore - a Cargo.toml fragment, not Rust
const NATIVE_CARGO: &str = r#"[dependencies]
dioxus ={ version = "0.8.0-alpha.1", features = ["native"] }
libero = { version = "0.1", features = ["native"] }

# Blitz is unusably slow unoptimised; this keeps your own crate debuggable.
[profile.dev.package."*"]
opt-level = 3

# `release` above is tuned for wasm size, the wrong trade for a window.
[profile.native]
inherits = "release"
opt-level = 3
lto = "thin"
codegen-units = 16"#;

// snippet: ignore - shell commands, not Rust
const NATIVE_COMMANDS: &str = "dx serve --platform linux --renderer native
dx build --platform linux --renderer native --profile native";

// snippet: ignore - shell commands, not Rust
const DESKTOP_COMMANDS: &str = "dx serve --platform desktop --renderer webview
dx build --platform desktop --renderer webview --release";

// snippet: ignore - shell commands, not Rust
const ANDROID_COMMANDS: &str = "rustup target add aarch64-linux-android x86_64-linux-android
export ANDROID_HOME=$HOME/Android/Sdk
dx serve --platform android
dx build --platform android";

// snippet: ignore - a Dioxus.toml fragment, not Rust
const ANDROID_DIOXUS: &str = r#"[android]
identifier = "com.example.app""#;

// snippet: ignore - shell commands, not Rust
const IOS_COMMANDS: &str = "dx serve --platform ios
dx build --platform ios";

/// Where the app runs, one tab each.
#[derive(Clone, Copy, PartialEq, Options)]
enum Environment {
    Web,
    #[option(label = "Native (Blitz)")]
    Native,
    #[option(label = "Desktop (WebView)")]
    Desktop,
    Android,
    #[option(label = "iOS")]
    Ios,
}

#[component]
pub fn GettingStarted() -> Element {
    let mut environment = use_signal(|| Environment::Web);

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
                CodeBlock { source: "cargo add libero", language: "shell" }
                Text {
                    "Wrap your app in "
                    Code { source: "LiberoProvider" }
                    " once, at the root. It registers the theme and every style your components use."
                }
                CodeBlock { source: QUICK_START_EXAMPLE, language: "rust" }
            }

            DocSection {
                title: "Build and run",
                Tabs {
                    aria_label: "Platform",
                    value: environment(),
                    onchange: move |next| environment.set(next),
                    panel: |environment: Environment| match environment {
                        Environment::Web => rsx! { WebPanel {} },
                        Environment::Native => rsx! { NativePanel {} },
                        Environment::Desktop => rsx! { DesktopPanel {} },
                        Environment::Android => rsx! { AndroidPanel {} },
                        Environment::Ios => rsx! { IosPanel {} },
                    },
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

/// A small list with Philosophy's check marks, so it does not read as a paragraph.
#[component]
fn Checklist(children: Element) -> Element {
    rsx! {
        List {
            size: "sm",
            icon: rsx! { Icon { variant: "standard", color: "primary", size: "sm", Pictogram { icon: lucide::check::outlined } } },
            {children}
        }
    }
}

/// A tab's body: a column with room under the strip.
#[component]
fn Panel(children: Element) -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", sx: sx().padding_top("md"), {children} }
    }
}

#[component]
fn WebPanel() -> Element {
    rsx! {
        Panel {
            CodeBlock { source: WEB_COMMANDS, language: "shell" }
            Text { "What keeps the download small:" }
            Checklist {
                ListItem {
                    "A release profile tuned for size. "
                    Code { source: "dx build --release" }
                    " runs "
                    Code { source: "wasm-opt" }
                    " on top, which "
                    Code { source: "dx" }
                    " downloads itself."
                }
                ListItem {
                    Code { source: "pre_compress" }
                    " in "
                    Code { source: "Dioxus.toml" }
                    " writes a brotli copy of the wasm and every asset beside it. Your host has to serve the "
                    Code { source: ".br" }
                    " files."
                }
                ListItem {
                    "Only the "
                    Code { source: "code-lang-*" }
                    " grammars your pages highlight."
                }
            }
            CodeBlock { source: WEB_PROFILE, language: "toml" }
            CodeBlock { source: WEB_DIOXUS, language: "toml" }
        }
    }
}

#[component]
fn NativePanel() -> Element {
    rsx! {
        Panel {
            Text {
                "A window drawn by Blitz, no browser involved. Turn on "
                Code { source: "native" }
                " in dioxus and in libero, and name your platform ("
                Code { source: "linux" }
                ", "
                Code { source: "macos" }
                " or "
                Code { source: "windows" }
                ")."
            }
            CodeBlock { source: NATIVE_CARGO, language: "toml" }
            CodeBlock { source: NATIVE_COMMANDS, language: "shell" }
            Text { "On Linux you need:" }
            Checklist {
                ListItem {
                    Code { source: "pkg-config" }
                    ", "
                    Code { source: "fontconfig" }
                    " and OpenSSL to build ("
                    Code { source: "pkg-config libfontconfig1-dev libssl-dev" }
                    " on Debian and Ubuntu)."
                }
                ListItem {
                    Code { source: "libxkbcommon" }
                    " and the Wayland or X11 client libraries to run. winit loads them itself, and a desktop session has them."
                }
                ListItem {
                    "A Vulkan driver to draw ("
                    Code { source: "mesa-vulkan-drivers" }
                    ", or "
                    Code { source: "vulkan-intel" }
                    " on Arch). Without a GPU, dioxus-native's CPU renderer ("
                    Code { source: "vello-cpu-softbuffer" }
                    ") still draws."
                }
            }
        }
    }
}

#[component]
fn DesktopPanel() -> Element {
    rsx! {
        Panel {
            Alert { title: "Early",
                Text {
                    "Focus traps and tree keys do nothing in a WebView yet, as on Android."
                }
            }
            Text {
                "A window with the system WebView inside: WebKitGTK on Linux, WebView2 on Windows, WKWebView on macOS. "
                "The pages look and behave like the web build. Turn on dioxus's "
                Code { source: "desktop" }
                " feature, and libero's "
                Code { source: "desktop" }
                " for system notifications on Linux."
            }
            CodeBlock { source: DESKTOP_COMMANDS, language: "shell" }
            Text { "On Linux you need:" }
            Checklist {
                ListItem {
                    "WebKitGTK 4.1, GTK 3 and xdo to build ("
                    Code { source: "libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev" }
                    " on Debian and Ubuntu, "
                    Code { source: "webkit2gtk-4.1 xdotool" }
                    " on Arch)."
                }
            }
        }
    }
}

#[component]
fn AndroidPanel() -> Element {
    rsx! {
        Panel {
            Alert { title: "Early",
                Text {
                    "Android runs, but it is the least tested target. Focus traps and tree keys do nothing there yet."
                }
            }
            Text {
                "The app runs in the system WebView, so it looks and behaves like the web build. Turn on dioxus's "
                Code { source: "mobile" }
                " feature; libero needs no feature of its own."
            }
            Text { "Install once:" }
            Checklist {
                ListItem { "The Android SDK with the NDK (27), platform 34 and build tools 34, and a JDK." }
                ListItem { "An emulator or a device that " Code { source: "adb devices" } " lists." }
            }
            CodeBlock { source: ANDROID_COMMANDS, language: "shell" }
            Text {
                Code { source: "dx" }
                " finds the NDK only through "
                Code { source: "ANDROID_HOME" }
                ". It regenerates the gradle project on every build, so set the app id in "
                Code { source: "Dioxus.toml" }
                ". A Rust panic shows only in "
                Code { source: "adb logcat" }
                "."
            }
            CodeBlock { source: ANDROID_DIOXUS, language: "toml" }
        }
    }
}

#[component]
fn IosPanel() -> Element {
    rsx! {
        Panel {
            Alert { title: "Untested",
                Text { "Nobody has run it locally or on a device; expect rough edges." }
            }
            Text {
                "The app runs in the system WebView (WKWebView), so it looks and behaves like the web build. Turn on dioxus's "
                Code { source: "mobile" }
                " feature; libero needs no feature of its own."
            }
            Text { "Install once:" }
            Checklist {
                ListItem { "macOS." }
                ListItem { "Xcode with the iOS Simulator runtime." }
                ListItem {
                    Code { source: "rustup target add aarch64-apple-ios-sim aarch64-apple-ios x86_64-apple-ios" }
                }
            }
            CodeBlock { source: IOS_COMMANDS, language: "shell" }
        }
    }
}
