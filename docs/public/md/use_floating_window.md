# use_floating_window

Crate: `libero`
Import: `use libero::hooks::use_floating_window;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_floating_window.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Registers a movable, non-modal window over the page and returns the handle that opens it.

`use_floating_window(options, render) -> FloatingWindowHandle` registers a
movable window over the page and returns the handle that opens it. The window
is not modal, so the page stays usable. [FloatingWindow](floating_window.md)
covers resizing, placement and the window menu.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, FloatingWindowOptions, Text},
    hooks::use_floating_window,
};

#[component]
fn Notes() -> Element {
    let notes = use_floating_window(
        FloatingWindowOptions {
            title: Some("Notes".into()),
            placement: "bottom-end".into(),
            ..Default::default()
        },
        |window| rsx! {
            Text { "Drag the title bar, or focus it and use the arrow keys." }
            Button { onclick: move |_| window.close(), "Done" }
        },
    );

    rsx! {
        Button { variant: "outlined", onclick: move |_| notes.toggle(), "Notes" }
    }
}
```

## Accessibility

The `title` names the window. It takes focus on open, and Escape, its close
button or `close()` hand focus back to the trigger. F6 moves between the page
and the window.

## API

```rust,ignore
pub fn use_floating_window(
    options: FloatingWindowOptions,
    render: impl FnMut(FloatingWindowHandle) -> Element + 'static,
) -> FloatingWindowHandle
```

| Method | Returns | Description |
|---|---|---|
| `open()` | `()` | Shows the window. Opening an open window does nothing. |
| `close()` | `()` | Hides it and returns focus to the trigger, unless focus has moved on. |
| `toggle()` | `()` | Opens or closes it. |
| `is_open()` | `bool` | Whether it is showing. |

`Copy`. `FloatingWindowOptions` is listed on
[FloatingWindow](floating_window.md).
