# Libero

A [Dioxus](https://dioxuslabs.com) component library focused on developer experience, UX, accessibility, and configurability.

Libero provides a themeable set of components (`Button`, `Flex`, `Select`, `Image`, `Backdrop`, `FocusTrap`, ...) styled through a small `sx` builder, so you get ergonomic, theme-aware styling without leaving Rust or reaching for a CSS framework.

## Example

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Title},
    sx::sx,
    LiberoProvider,
};

fn app() -> Element {
    rsx! {
        LiberoProvider {
            Flex {
                direction: "column",
                gap: "md",
                sx: sx().padding_top("md"),
                Title { variant: "h1", "Welcome to Libero" }
                Button {
                    variant: "filled",
                    color: "primary",
                    onclick: move |_| println!("clicked"),
                    "Get started"
                }
            }
        }
    }
}
```

## Features

All additive. The default set is five `code-lang-*` highlighter grammars
(Rust, Bash, Markdown, HTML and CSS).

- `code-lang-*` - one hand-ported grammar each for `Code` and `CodeBlock`, 30
  in all. With `default-features = false` you compile only the ones you name.
- `full-polymorphism` - `Box`'s `component` prop accepts all 111 HTML5 element
  names, and 83 of them (every sectioning, text-level, list, table and form
  element) render on default features. This adds the remaining 28: document
  metadata, embedded and media content, `template`/`slot`, and the bidi and
  ruby set. Without it those render as a `div`, silently in a release build.
- `native` - reach elements through Blitz when running under `dioxus-native`,
  instead of Dioxus's portable mounted handle.

## Status

Libero is a work in progress. APIs may change between `0.x` releases.

## License

MIT OR Apache-2.0
