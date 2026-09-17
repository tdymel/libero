# use_clipboard

Crate: `libero`
Import: `use libero::hooks::{Clipboard, use_clipboard};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/clipboard.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Writes text to the system clipboard and reports whether the write worked.

`use_clipboard() -> Clipboard` writes text to the system clipboard. `copy(text)`
starts the write, and `copied()` turns true once the platform confirms it. A
refused write raises `failed()` instead and logs a warning. Either flag stays up
until you call `reset()`. `Clipboard` is `Copy`, so handlers take it without a
clone.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::Button, hooks::use_clipboard};

#[component]
fn CopyLink() -> Element {
    let mut clipboard = use_clipboard();

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| clipboard.copy("https://github.com/tdymel/libero"),
            onblur: move |_| clipboard.reset(),
            "Copy link"
        }
        // Mounted before it has anything to say, so the change is announced.
        span { role: "status",
            if clipboard.copied() { "Copied" } else if clipboard.failed() { "Copy failed" }
        }
    }
}
```

## Accessibility

Say the result in a status region that is already mounted. A button whose own
label changes is not announced.

## Web and native

On the web the browser allows the write only in a secure context (HTTPS or
localhost) and inside a user action, such as the click handler above. Natively
(the `native` feature) it writes the system clipboard. A desktop build without
that feature, such as a webview, has no clipboard, so `copy` raises `failed()`
at once.

## API

```rust,ignore
pub fn use_clipboard() -> Clipboard
```

| Method | Returns | Description |
|---|---|---|
| `copy(text: impl Into<String>)` | `()` | Starts the write. `copied()` or `failed()` rises once the platform answers. |
| `copied()` | `bool` | Whether the last write succeeded and nothing has reset it. Reactive. |
| `failed()` | `bool` | Whether the last write was refused, or there is no clipboard. Reactive. |
| `reset()` | `()` | Clears both flags. |
