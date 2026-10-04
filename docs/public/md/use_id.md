# Unique ID

Crate: `libero`
Import: `use libero::hooks::use_id;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/id.rs>
Index: [index.md](index.md) lists every other page
Description: An id unique within the app, stable for the component's lifetime, for the aria wiring between one instance's elements.

`use_id() -> Signal<String>` returns an id that is unique within the app and
stays the same for the component's lifetime. Use it for the aria wiring between
one instance's elements, where a fixed string would clash as soon as the
component renders twice.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Text},
    hooks::use_id,
};

#[component]
fn Disclosure(title: String, children: Element) -> Element {
    let panel = use_id();
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            variant: "standard",
            aria_expanded: open(),
            aria_controls: panel(),
            onclick: move |_| open.toggle(),
            "{title}"
        }
        Box { id: panel(), hidden: !open(), {children} }
    }
}

#[component]
fn Faq() -> Element {
    rsx! {
        Disclosure { title: "Shipping", Text { "Two to four working days." } }
        Disclosure { title: "Returns", Text { "Free within 30 days." } }
    }
}
```

## API

```rust,ignore
pub fn use_id() -> Signal<String>
```

The id reads `lsx-N`, from a counter per app that follows render order, so a
server render and its hydration agree. Read it with `id()` where an attribute
wants a `String`.

## Accessibility

### Libero handles

- The id is unique within the app, so each instance's wiring stays its own.
  Each disclosure in the demo names its own panel, so a screen reader pairs
  every button with the right one.

### You must

- Pass the id to `aria_controls`, `aria_labelledby`, `aria_describedby` or a
  label's `r#for`: an id is how they find their element.

### Example

A disclosure in an FAQ: the button takes `aria_controls: id()` and the panel
`id: id()`, so each question opens and names its own answer, however many
disclosures the page has.
