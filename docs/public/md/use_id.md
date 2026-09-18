# use_id

Crate: `libero`
Import: `use libero::hooks::use_id;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/id.rs>
Index: [index.md](index.md) lists every other page
Description: A process-unique id, stable for the component's lifetime, for the aria wiring between one instance's elements.

`use_id() -> Signal<String>` returns an id that is unique in the process and
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

## Accessibility

An id is how `aria_controls`, `aria_labelledby`, `aria_describedby` and a
label's `r#for` find their element. Each disclosure above names its own panel,
so a screen reader pairs every button with the right one.

## API

```rust,ignore
pub fn use_id() -> Signal<String>
```

The id reads `lsx-N`, from a counter shared by the whole process. Read it with
`id()` where an attribute wants a `String`.
