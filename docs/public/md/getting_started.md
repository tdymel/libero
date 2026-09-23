# Getting started

Crate: `libero`
Import: `use libero::LiberoProvider;`
Index: [index.md](index.md) lists every other page
Description: Installing libero, wrapping an app in LiberoProvider, building for the web, natively, in a desktop WebView and for Android, and the feature flags.

Libero is a Dioxus component library focused on developer experience, UX,
accessibility, and configurability.

## Installation

```shell
cargo add libero
```

Wrap your app in `LiberoProvider` once, at the root. It registers the theme and
every style your components use.

```rust
use dioxus::prelude::*;
use libero::{LiberoProvider, components::Text};

fn App() -> Element {
    rsx! {
        LiberoProvider {
            Text { "Hello, Libero!" }
        }
    }
}
```

Without it, components render but carry no theme and no stylesheet, so nothing
is styled.

## Build and run

### Web

```shell
dx serve --platform web
dx build --platform web --release
```

What keeps the download small:

- A release profile tuned for size. `dx build --release` runs `wasm-opt` on
  top, which `dx` downloads itself.
- `pre_compress` in `Dioxus.toml` writes a brotli copy of the wasm and every
  asset beside it. Your host has to serve the `.br` files.
- Only the `code-lang-*` grammars your pages highlight.

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

```toml
[web]
pre_compress = true
```

### Native (Blitz)

A window drawn by Blitz, no browser involved. Turn on `native` in dioxus and in
libero, and name your platform (`linux`, `macos` or `windows`).

```toml
[dependencies]
dioxus = { version = "0.8.0-alpha.1", features = ["native"] }
libero = { version = "0.1", features = ["native"] }

# Blitz is unusably slow unoptimised; this keeps your own crate debuggable.
[profile.dev.package."*"]
opt-level = 3

# `release` above is tuned for wasm size, the wrong trade for a window.
[profile.native]
inherits = "release"
opt-level = 3
lto = "thin"
codegen-units = 16
```

```shell
dx serve --platform linux --renderer native
dx build --platform linux --renderer native --profile native
```

On Linux you need:

- `pkg-config`, `fontconfig` and OpenSSL to build
  (`pkg-config libfontconfig1-dev libssl-dev` on Debian and Ubuntu).
- `libxkbcommon` and the Wayland or X11 client libraries to run. winit loads
  them itself, and a desktop session has them.
- A Vulkan driver to draw (`mesa-vulkan-drivers`, or `vulkan-intel` on Arch).
  Without a GPU, dioxus-native's CPU renderer (`vello-cpu-softbuffer`) still
  draws.

### Desktop (WebView)

Early: focus traps and tree keys do nothing in a WebView yet, as on Android.

A window with the system WebView inside: WebKitGTK on Linux, WebView2 on
Windows, WKWebView on macOS. The pages look and behave like the web build. Turn
on dioxus's `desktop` feature; libero needs no feature of its own.

```shell
dx serve --platform desktop --renderer webview
dx build --platform desktop --renderer webview --release
```

On Linux you need:

- WebKitGTK 4.1, GTK 3 and xdo to build
  (`libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev` on Debian and Ubuntu,
  `webkit2gtk-4.1 xdotool` on Arch).

### Android

Early: Android runs, but it is the least tested target. Focus traps and tree
keys do nothing there yet.

The app runs in the system WebView, so it looks and behaves like the web build.
Turn on dioxus's `mobile` feature; libero needs no feature of its own.

Install once:

- The Android SDK with the NDK (27), platform 34 and build tools 34, and a JDK.
- An emulator or a device that `adb devices` lists.

```shell
rustup target add aarch64-linux-android x86_64-linux-android
export ANDROID_HOME=$HOME/Android/Sdk
dx serve --platform android
dx build --platform android
```

`dx` finds the NDK only through `ANDROID_HOME`. It regenerates the gradle
project on every build, so set the app id in `Dioxus.toml`. A Rust panic shows
only in `adb logcat`.

```toml
[android]
identifier = "com.example.app"
```

### iOS

Untested: nobody has run it locally or on a device; expect rough edges.

The app runs in the system WebView (WKWebView), so it looks and behaves like
the web build. Turn on dioxus's `mobile` feature; libero needs no feature of
its own.

Install once:

- macOS.
- Xcode with the iOS Simulator runtime.
- `rustup target add aarch64-apple-ios-sim aarch64-apple-ios x86_64-apple-ios`

```shell
dx serve --platform ios
dx build --platform ios
```

## Feature flags

Every feature is additive.

| Flag | What it does |
|---|---|
| `code-lang-<name>` | One grammar for [Code](code.md) and [CodeBlock](code_block.md), 30 in all. Rust, Bash, Markdown, HTML and CSS are the default. |
| `full-polymorphism` | [Box](box.md) renders the rarer HTML elements too (metadata, media, web components). Without it they fall back to a `div`. |
| `native` | Element access through Blitz, for apps on `dioxus-native`. |

```toml
libero = { version = "0.1", default-features = false, features = [
    "code-lang-rust",
    "full-polymorphism",
] }
```
