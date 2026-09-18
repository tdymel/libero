# use_modal

Crate: `libero`
Import: `use libero::hooks::use_modal;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_modal.rs>
Index: [index.md](index.md) lists every other page
Description: Registers a modal and returns the handle that opens it, with arguments and a result.

`use_modal(render) -> ModalHandle<S, R>` registers a modal and returns the
handle that opens it. The modal can take arguments `S` and answer with a result
`R`, which the caller reads with `onresult` or `.await`. [Modal](modal.md) has
the full story.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, Text},
    hooks::{ModalScope, use_modal},
};

#[component]
fn DeleteFile() -> Element {
    let confirm = use_modal(|s: ModalScope<(), bool>| rsx! {
        Dialog { title: "Delete notes.md?", size: "sm",
            Text { "This cannot be undone." }
            Button { variant: "text", onclick: move |_| s.close(), "Cancel" }
            Button { variant: "filled", color: "error", onclick: move |_| s.resolve(true), "Delete" }
        }
    });
    let mut answer = use_signal(|| "nothing yet");

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                confirm.open().onresult(move |deleted| {
                    answer.set(if deleted == Some(true) { "deleted" } else { "kept" });
                });
            },
            "Delete file"
        }
        Text { "Last answer: {answer}" }
    }
}
```

A dismissal answers `None`, so Escape or a backdrop click reads as "kept"
here. Call the hook in a component that outlives every trigger, because the
modal is portaled from there.

## Accessibility

The modal traps focus while it is open, and focus goes back to the button that
opened it when it closes. Open it from the handler of what the user acted on,
so that is the element remembered. The `Dialog` inside supplies the role, and
its `title` is the accessible name.

## API

```rust,ignore
pub fn use_modal<S: Clone + 'static, R: Clone + 'static>(
    render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
```

| `ModalHandle` method | Returns | Description |
|---|---|---|
| `open()` | `Opening<R>` | Opens with `S::default()`. |
| `open_with(args: impl Into<S>)` | `Opening<R>` | Opens with `args`, superseding what it was showing. |
| `close()` | `()` | Closes it as a dismissal. |
| `is_open()` | `bool` | Whether it is showing. |

`Opening<R>` takes `.onresult(|Option<R>| ...)` or is awaited. Inside,
`ModalScope` has `args()`, `resolve(value)` and `close()`. `ModalHandle` is
`Copy`.
