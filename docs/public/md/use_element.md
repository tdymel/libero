# use_element

Crate: `libero`
Import: `use libero::{hooks::{ElementHandle, use_element}, platform::ElementApi};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/element.rs>
Index: [index.md](index.md) lists every other page
Description: A handle to one of your component's own elements, to focus, scroll and measure it on every renderer.

`use_element() -> ElementHandle` is a handle to one of your component's own
elements, and the only way to reach an element at all. Mount it with
`onmounted: handle.mount()`. The handle is `Copy` and implements `ElementApi`,
so it focuses, scrolls and measures the same way on every renderer.

Commands such as `focus()` return at once. Reads such as `dimensions()` are
futures. Start one in the handler and await it in a `spawn`. Until the element
mounts, every call answers `PlatformError::Unsupported`. `is_mounted()` is
reactive, so an effect that reads it runs again once the element is there. The
full method list is on [Platform](platform.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button},
    hooks::use_element,
    platform::ElementApi,
    sx::sx,
};

#[component]
fn Measure() -> Element {
    let panel = use_element();
    let mut size = use_signal(String::new);

    rsx! {
        Box {
            onmounted: panel.mount(),
            sx: sx().width("240px").height("96px").padding("md")
                .background("muted.1").border_radius("8px"),
            style: "resize: both; overflow: auto",
            "Drag the corner to resize me."
        }
        Button {
            variant: "outlined",
            onclick: move |_| {
                // Start the read in the handler, await it in a task.
                let read = panel.dimensions();
                spawn(async move {
                    if let Ok(box_size) = read.await {
                        size.set(format!("{:.0} × {:.0} px", box_size.width, box_size.height));
                    }
                });
            },
            "Measure"
        }
        span { role: "status", "{size}" }
    }
}
```

## Web and native

The handle picks the richest backing the renderer offers: the DOM element on the
web, a Blitz node natively (the `native` feature), and dioxus's portable mounted
data in a webview. A webview still measures, scrolls and focuses, but
`query_selector` answers `Unsupported` there, and `is_focused()` always answers
`false`.

## API

```rust,ignore
pub fn use_element() -> ElementHandle
```

| Method | Returns | Description |
|---|---|---|
| `mount()` | `impl FnMut(Event<MountedData>)` | The `onmounted` handler that fills the handle in. |
| `is_mounted()` | `bool` | Reactive: an effect reading it re-runs once the element mounts. |

`ElementHandle` is `Copy` and `PartialEq`, and implements `ElementApi`. See
[platform.md](platform.md) for its methods.
