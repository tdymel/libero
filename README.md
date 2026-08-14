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

## Status

Libero is a work in progress. APIs may change between `0.x` releases.

## License

MIT OR Apache-2.0
