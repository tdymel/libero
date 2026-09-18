# use_stylesheet

Crate: `libero`
Import: `use libero::hooks::use_stylesheet;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/stylesheet.rs>
Index: [index.md](index.md) lists every other page
Description: Registers a stylesheet of your own above every libero layer and returns its class.

`use_stylesheet(sheet) -> Option<String>` registers a stylesheet of your own on
the `lsx-user-custom` layer, above every other libero layer. Give it an `sx`
and it returns the class to put on your element. [Styling](styling.md)
explains the layers.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::Box, hooks::use_stylesheet, sx::sx};

#[component]
fn Callout(children: Element) -> Element {
    let class = use_stylesheet(&sx().background("primary.1").padding("md").border_radius("md"));

    rsx! {
        Box { class: class.unwrap_or_default(), {children} }
    }
}
```

Raw CSS as a `&str` or a `String` works too. It has no single selector, so it
returns `None`. Components that register the same sheet share one copy of it.

## API

```rust,ignore
pub fn use_stylesheet(stylesheet: impl Into<Stylesheet>) -> Option<String>
```

`Stylesheet` converts from `&Sx` and `&StaticSx`, which return a class name,
and from `&str` and `String`, which return `None`.
