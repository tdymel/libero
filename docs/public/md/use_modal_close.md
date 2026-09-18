# use_modal_close

Crate: `libero`
Import: `use libero::hooks::use_modal_close;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_modal.rs>
Index: [index.md](index.md) lists every other page
Description: Closes the modal it is rendered in, for a component factored out of the render closure.

`use_modal_close() -> Callback<()>` closes the modal it is rendered in. It is
for a component factored out of the [use_modal](use_modal.md) closure, which
cannot capture the `ModalScope`. [Modal](modal.md) has the full story.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, Text},
    hooks::{ModalScope, use_modal, use_modal_close},
};

/// Factored out of the render closure, so it cannot capture the `ModalScope`.
#[component]
fn DoneButton() -> Element {
    let close = use_modal_close();

    rsx! {
        Button { variant: "filled", onclick: move |_| close.call(()), "Done" }
    }
}

#[component]
fn SaveNotes() -> Element {
    let saved = use_modal(|_: ModalScope<()>| rsx! {
        Dialog { title: "Saved", size: "sm",
            Text { "notes.md is saved." }
            DoneButton {}
        }
    });

    rsx! {
        Button { variant: "outlined", onclick: move |_| { saved.open(); }, "Save" }
    }
}
```

Closing this way is a dismissal, the same as Escape. A component that may
render outside a modal asks first with `try_use_context::<ModalContext>()`, as
the Modal page shows.

## API

```rust,ignore
pub fn use_modal_close() -> Callback<()>
```

Call it as `close.call(())`.
