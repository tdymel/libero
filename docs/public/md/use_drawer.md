# use_drawer

Crate: `libero`
Import: `use libero::hooks::use_drawer;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_drawer.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Registers a panel docked to one edge and returns the handle that opens it.

`use_drawer(options, render) -> ModalHandle<S, R>` registers a panel docked to
one edge and returns the handle that opens it. It is [use_modal](use_modal.md)
with the docking around it, so it has the same handle, arguments and results.
[Drawer](drawer.md) has the full story.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Text, Title},
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

#[component]
fn FilterPanel() -> Element {
    let filters = use_drawer(
        DrawerOptions {
            anchor: "end".into(),
            size: "sm".into(),
            aria_label: Some("Filters".into()),
            ..Default::default()
        },
        |s: ModalScope<()>| rsx! {
            Title { size: "lg", "Filters" }
            Text { "Nothing to filter yet." }
            Button { variant: "text", onclick: move |_| s.close(), "Close" }
        },
    );

    rsx! {
        Button { variant: "outlined", onclick: move |_| { filters.open(); }, "Filters" }
    }
}
```

## Accessibility

The panel is a dialog with no name of its own, so set `aria_label`. It has no
header close button either, so the content brings its own. Like every modal it
traps focus, closes on Escape and hands focus back to its trigger.

## API

```rust,ignore
pub fn use_drawer<S: Clone + 'static, R: Clone + 'static>(
    options: DrawerOptions,
    render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
```

`DrawerOptions` has `anchor` (`start` by default), `size` (`md`), `z_index` and
`aria_label`. The handle is the one [use_modal](use_modal.md) returns.
