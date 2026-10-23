# Getting started

Crate: `libero`
Import: `use libero::LiberoProvider;`
Index: [index.md](index.md) lists every other page
Description: Installing libero, wrapping an app in LiberoProvider, building for the web, natively, in a desktop WebView and for Android, and the feature flags.

Libero is a Dioxus component library focused on developer experience, UX,
accessibility, and configurability.

## Installation

Libero 0.2 pins Dioxus `=0.8.0-alpha.3`, so add that exact version: a second
copy of dioxus would not share libero's contexts.

```shell
cargo add dioxus@=0.8.0-alpha.3
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
dioxus = { version = "=0.8.0-alpha.3", features = ["native"] }
libero = { version = "0.3", features = ["native"] }

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
on dioxus's `desktop` feature, and libero's `desktop` for system notifications
on Linux.

```shell
dx serve --platform desktop --renderer webview
dx build --platform desktop --renderer webview --release
```

On Linux you need:

- WebKitGTK 4.1 and GTK 3 to build
  (`libwebkit2gtk-4.1-dev libgtk-3-dev` on Debian and Ubuntu,
  `webkit2gtk-4.1` on Arch).

On Windows dioxus serves the page from `http://dioxus.index.html`, which
WebView2 does not count as a secure context: the camera, microphone, location,
notifications and clipboard APIs are missing. Mark that one origin secure,
never a wildcard or a list: it is your app's own bundled page. Keep wry's
default arguments, which these replace. Confirmed on Windows CI, where the
page then reports a secure context:

```rust,ignore
// wry 0.55's defaults, then the app's own page origin.
const WEBVIEW2_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
    --unsafely-treat-insecure-origin-as-secure=http://dioxus.index.html";

fn main() {
    let config = dioxus::desktop::Config::new().with_windows_browser_args(WEBVIEW2_ARGS);
    dioxus::LaunchBuilder::new().with_cfg(config).launch(App);
}
```

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

Simulator only: CI runs it in an iOS simulator; no real device has run it (see
What CI checks).

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

dioxus's page has no `viewport-fit=cover`, so the safe-area insets read 0 and
Header, NavigationBar and Lightbox run under the notch and the home
indicator. It also sets `maximum-scale=1, user-scalable=no`, which blocks pinch
zoom (WCAG 1.4.4); fields keep their text at 16px on iOS, so the page does not
need it. Rewrite the viewport meta from the head:

```rust,ignore
const VIEWPORT: &str = "<script>document.querySelector('meta[name=viewport]')\
    ?.setAttribute('content', 'width=device-width, initial-scale=1, viewport-fit=cover');</script>";

fn main() {
    let config = dioxus::mobile::Config::new().with_custom_head(VIEWPORT.into());
    dioxus::LaunchBuilder::new().with_cfg(config).launch(App);
}
```

## What CI checks

The Platforms workflow builds the docs app for iOS, macOS and Windows and runs
the e2e scenarios on each, informational and not part of the required check. A
scenario that flakes is fixed or skipped. 667 scenarios exist; the last
column counts the ones left out on that platform, besides 24 that exist for
Android only.

| Platform | CI runs | Skipped |
|---|---|---|
| iOS (WKWebView) | The simulator build, and the e2e suite in an iPhone simulator, with taps and keys injected through AXe. | 146: 120 because the simulator's tap, drag or Tab did not drive the page, 20 because a touch screen has no hover or wheel. |
| macOS desktop (WKWebView) | The desktop build, and the e2e suite in a WKWebView window, with the app sending its own NSEvents. | 129: 86 shared with every desktop WebView (the Tab bridge, element identity, touch input) and 43 only here, mostly WebKit not focusing a clicked button and WKWebView misreading list clicks, a fixed bar and wheel scrolls. |
| macOS native (Blitz) | The Blitz build, and the e2e suite on a windowless CPU renderer. | 57: 55 shared with Blitz on Linux (charts, right-to-left scrolling, media), a panic on a windowed table and a doubled Backspace. |
| Windows desktop (WebView2) | The desktop build, and the e2e suite in a WebView2 window, driven over its DevTools port. | 92: 86 shared with every desktop WebView and 6 only here (End in a tall listbox, Shift+Tab out of a date input, an early drop's focus, a picture's asset path). |
| Windows native (Blitz) | The Blitz build, and the e2e suite on a windowless CPU renderer. | 56: 55 shared with Blitz on Linux and a panic on a windowed table. |

No real device has run libero: iOS is a simulator, macOS and Windows are hosted
runners. On all three, no system permission prompt is answered except the iOS
location dialog, so a camera, a position or a notification on a device is
untested. Each run also records what every WebView reports (secure context,
`Notification`, fullscreen, capture), and the platform tables of the hooks quote
it.

## Feature flags

Every feature is additive.

| Flag | What it does |
|---|---|
| `<component>`, `use-<hook>`, `<group>` | One feature per component and per hook, and one per group that turns on its members. The default turns on the layout and the common components and hooks; see [Features](features.md). |
| `code-lang-<name>` | One grammar for [Code](code.md) and [CodeBlock](code_block.md), 30 in all, none on by default. |
| `collation` | [Table](table.md) sorts text by the app's language on desktop, Android and native too, with ICU4X's collation data (about 1.2 MB of binary). Without it the web sorts by language and the rest by accent-folded letters. |
| `full-polymorphism` | [Box](box.md) renders the rarer HTML elements too (metadata, media, web components). Without it they fall back to a `div`. |
| `native` | Element access through Blitz, for apps on `dioxus-native`. |
| `desktop` | System notifications over D-Bus in a desktop WebView on Linux, where WebKitGTK denies them. |
| `icons-bootstrap` | The Bootstrap Icons set for [IconProvider](icon_provider.md), as `IconSet::bootstrap()`. Without it libero's glyphs are Lucide. |
| `icons-material` | The Material Design Icons set for [IconProvider](icon_provider.md), as `IconSet::material()`. Without it libero's glyphs are Lucide. |
| `icons-phosphor` | The Phosphor set for [IconProvider](icon_provider.md), as `IconSet::phosphor()`. Without it libero's glyphs are Lucide. |
| `icons-tabler` | The Tabler Icons set for [IconProvider](icon_provider.md), as `IconSet::tabler()`. Without it libero's glyphs are Lucide. |

```toml
libero = { version = "0.3", default-features = false, features = [
    "code-lang-rust",
    "full-polymorphism",
] }
```
