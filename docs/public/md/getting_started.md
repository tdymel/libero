# Getting started

Crate: `libero`
Import: `use libero::LiberoProvider;`
Index: [index.md](index.md) - every component's markdown page
Description: Installing libero, wrapping an app in LiberoProvider, and the feature flags a web build wants.

Libero is a Dioxus component library focused on developer experience, UX,
accessibility, and configurability.

## Installation

Add Libero to your project with cargo:

```shell
cargo add libero
```

## Quick start

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

## Building for the web

Dioxus's `wasm-split` feature puts every route in its own chunk, fetched on its
first visit, so the initial bundle stays small. On this docs site the
brotli-compressed main bundle is 297 KB instead of 414 KB. Libero adds no split
points of its own. The per-route chunks already carry [Code](code.md)'s
highlighter and [QrCode](qr_code.md)'s encoder to the pages that use them.

```toml
dioxus = { version = "0.8.0-alpha.1", features = ["router", "wasm-split"] }
```

It is experimental, and `dx` enables it only when asked. With the feature on,
always build and serve with `--wasm-split`, or the app fails to load.

```shell
dx serve --platform web --release --debug-symbols=false --wasm-split
```

Without it, drop the feature and the `--wasm-split` flag. The app renders the
same, from one bundle. Either way, keep only the languages your own examples
use:

```toml
libero = { version = "0.1", default-features = false, features = ["code-lang-rust"] }
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
