# Stylesheet

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

## Raw CSS

Raw CSS as a `&str` or a `String` works too. It has no single selector, so it
returns `None`. Components that register the same sheet share one copy of it.

## Accessibility

### Libero handles

- Nothing on screen: it registers CSS and returns a class.

### You must

- Keep a visible focus indicator: a rule here outranks libero's own focus ring,
  so `outline: none` on a control removes it (WCAG 2.4.7).
- Check the contrast of a literal colour you set: 4.5:1 for text, 3:1 for
  borders and icons (WCAG 1.4.3, 1.4.11). A theme colour such as `primary.1`
  follows the theme set.

### Example

A card class from `use_stylesheet` that gives its link `color: primary.6` and
leaves the focus ring alone: the link keeps libero's visible ring, and the
theme colour follows the theme set.

## API

```rust,ignore
pub fn use_stylesheet(stylesheet: impl Into<Stylesheet>) -> Option<String>
```

`Stylesheet` converts from `&Sx` and `&StaticSx`, which return a class name,
and from `&str` and `String`, which return `None`.
