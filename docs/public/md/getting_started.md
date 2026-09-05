# Getting Started

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

## Quick Start

Wrap your app in `LiberoProvider` once, at the root - it registers the theme and
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

## Building for the Web

Dioxus's `wasm-split` feature puts every route in its own chunk, fetched when it
is first visited instead of bloating every page's initial bundle. On this docs
site that is 297 KB of brotli-compressed main bundle instead of 414 KB. Libero
adds no split points of its own - the per-route chunks already carry
[Code](code.md)'s highlighter and [QrCode](qr_code.md)'s encoder to the pages
that use them.

```toml
dioxus = { version = "*", features = ["router", "wasm-split"] }
```

It is experimental, and `dx` only enables it when asked - with the feature on,
always build and serve with `--wasm-split`, or the app will fail to load
entirely (a dangling module import, not a graceful fallback).

```shell
dx serve --platform web --release --debug-symbols=false --wasm-split
```

Don't need it? Drop the feature and skip `--wasm-split` entirely - the app
renders identically either way, just from one bundle instead of per-route
chunks. Either way, keep only the languages your own examples use:

```toml
libero = { version = "*", default-features = false, features = ["code-lang-rust"] }
```

## Feature flags

Every libero feature is additive, and the default set is five `code-lang-*`
grammars: Rust, Bash, Markdown, HTML and CSS. Two are worth knowing about.

`code-lang-*` compiles one hand-ported grammar each for [Code](code.md) and
[CodeBlock](code_block.md). There are 30 of them; with
`default-features = false` you pay only for the ones you name.

`full-polymorphism` widens what [Box](box.md)'s `component` prop can render.
All 111 HTML5 element names are accepted and type-check either way, but only 83
of them compile a match arm by default: every sectioning, text-level, list,
table and form element - `footer`, `strong`, `em`, `small`, `time`, `details`,
`dialog` and the rest. The feature adds the remaining 28:

| Family | Tags |
|---|---|
| Document metadata | `head`, `meta`, `title`, `script`, `style`, `link`, `base`, `body`, `noscript` |
| Embedded and media content | `iframe`, `canvas`, `audio`, `video`, `picture`, `source`, `track`, `embed`, `object`, `param`, `map`, `area` |
| Web components | `template`, `slot` |
| Bidi and ruby annotation | `bdi`, `bdo`, `ruby`, `rp`, `rt` |

Pass one of those 28 without the feature and you get a `div` - with a console
warning in a debug build, and silently in a release build, because the warning
compiles to nothing there. On this docs site the feature costs 14.8 KB of wasm,
1.0 KB after brotli.

```toml
libero = { version = "*", default-features = false, features = [
    "code-lang-rust",
    "full-polymorphism",
] }
```

`native` is the third: it reaches elements through Blitz when you run under
`dioxus-native`, instead of Dioxus's portable mounted handle, which cannot
query a subtree or report focus.

## Where to go next

[Styling](styling.md), [Theming](theming.md) and
[Performance](performance.md) cover how the library works;
[index.md](index.md) lists one page per component.
