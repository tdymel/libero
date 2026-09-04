# Modal

Crate: `libero`
Import: `use libero::hooks::{ModalHandle, ModalScope, use_modal};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/modal.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A modal is a hook, not a component - `use_modal` registers a render closure and returns a handle that opens it, with per-opening arguments, results and handlers.

A modal is a hook, not a component. `use_modal` registers a render closure and
hands back a `ModalHandle` that opens it - no open flag to thread, no
conditional branch at the call site, and the content is built only while it is
showing.

There is no public `Modal` component. Pair the hook with [`Dialog`](dialog.md),
which supplies the role, the accessible name and its own close button.

## Usage

Wrap `use_modal` in a hook of your own: its parameters are shared by every
opening, the `ModalScope` carries the arguments of the one being shown, and
`close()` / `resolve(value)` end it.

Make the result the dialog's own enum, not a `bool` - the caller then matches
over what it can say, with a dismissal as `None` beside it.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, Text},
    hooks::{ModalHandle, ModalScope, use_modal},
};

/// What the dialog can answer. A dismissal answers nothing, so the caller
/// matches on `Option<SaveChoice>` and "went back" is a case like any other.
#[derive(Clone, Copy, PartialEq)]
enum SaveChoice {
    Save,
    Discard,
}

/// `discard_label` is shared by every opening - the closure just captures it.
fn use_save_prompt(discard_label: &'static str) -> ModalHandle<String, SaveChoice> {
    use_modal(move |s: ModalScope<String, SaveChoice>| {
        let document = s.args();

        rsx! {
            Dialog {
                title: "Unsaved changes",
                size: "sm",
                Text { "{document} has changes you have not saved." }
                Button { variant: "text", onclick: move |_| s.close(), "Keep editing" }
                Button {
                    variant: "filled",
                    color: "error",
                    onclick: move |_| s.resolve(SaveChoice::Discard),
                    "{discard_label}"
                }
                Button {
                    variant: "filled",
                    color: "primary",
                    onclick: move |_| s.resolve(SaveChoice::Save),
                    "Save"
                }
            }
        }
    })
}

#[component]
fn Demo() -> Element {
    let prompt = use_save_prompt("Discard");

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                prompt.open_with("notes.md").on_result(move |answer| match answer {
                    Some(SaveChoice::Save) => { /* save it */ }
                    Some(SaveChoice::Discard) => { /* throw it away */ }
                    None => {}   // dismissed - keep editing
                });
            },
            "Close editor"
        }
    }
}
```

## Openings

`open_with` takes anything that converts into the argument type and returns an
`Opening` - that one showing. A later opening supersedes it, and a superseded
`Opening` is inert: it can neither fire its handler nor close what is on screen
now, and it awaits to `None`.

```rust
// Attach the consequence to this one opening.
prompt.open_with("notes.md").on_result(move |answer| { /* ... */ });

// Awaited instead - for an answer that gates work which is already async, or
// several dialogs in sequence.
match prompt.open_with("notes.md").await {
    Some(SaveChoice::Save) => save().await,
    Some(SaveChoice::Discard) => discard().await,
    None => return,
}

// Arguments skipped, when the argument type is `Default`.
prompt.open();

// Held and closed later.
let opening = prompt.open_with("notes.md");
opening.close();

// Closes whatever this modal is currently showing.
prompt.close();
```

## Where to call it

`use_modal` must be called under `LiberoProvider`, in a component that outlives
every trigger - the modal is portaled from there and unmounts with it. A
component that needs a dialog simply calls the hook itself.

The handle is `Copy`, so a trigger elsewhere in the tree can take it as a prop.
For one shared instance behind several triggers, provide it from your own hook:

```rust
fn use_save_prompt(discard_label: &'static str) -> ModalHandle<String, SaveChoice> {
    let handle = use_modal(/* ... */);
    use_context_provider(|| handle);
    handle
}
```

## Closing from inside

Content written in the render closure captures the `ModalScope`. A `Dialog`
inside a modal also closes itself from its own header button, with no wiring.
Only a component factored out of the closure needs the escape hatch:

```rust
use dioxus::prelude::*;
use libero::{components::Button, hooks::use_modal_close};

#[component]
fn CancelButton() -> Element {
    let close = use_modal_close();

    rsx! {
        Button { variant: "text", onclick: move |_| close.call(()), "Cancel" }
    }
}
```

## Accessibility

Escape or a backdrop click dismisses the modal, settling the `Opening` with
`None`, so a handler written for an answer never runs on a dismissal. Name the
`Dialog` inside with its `title`, or `aria_label`.

## API

### `use_modal`

```rust
pub fn use_modal<S: Clone + 'static, R: Clone + 'static>(
    render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
```

`R` defaults to `()`, so a modal that answers nothing is `ModalScope<S>`.

### `ModalHandle<S, R = ()>`

| Method | Returns | Description |
|---|---|---|
| `open_with(args: impl Into<S>)` | `Opening<R>` | Opens, superseding whatever was showing. |
| `open()` | `Opening<R>` | Opens with `S::default()`; needs `S: Default`. |
| `close()` | `()` | Dismisses whatever is currently showing. |
| `is_open()` | `bool` | Whether this modal is showing. |

`Copy`, so it can be passed to a trigger elsewhere in the tree.

### `ModalScope<S, R = ()>`

| Method | Returns | Description |
|---|---|---|
| `args()` | `S` | The arguments this opening was given. |
| `close()` | `()` | Ends it as a dismissal - the same outcome as Escape. |
| `resolve(value: R)` | `()` | Ends it with an answer for the caller. |

`Copy`, so several handlers in one render closure can each hold it.

### `Opening<R = ()>`

| Method | Returns | Description |
|---|---|---|
| `on_result(f: impl FnMut(Option<R>))` | `Self` | Runs `f` when this opening settles; `None` if it was dismissed. |
| `close()` | `()` | Closes this opening, if it is still the one showing. |
| `.await` | `Option<R>` | Same outcome, as a future. |

`Copy`, and inert once superseded.

### `use_modal_close`

```rust
pub fn use_modal_close() -> Callback<()>
```

Closes the modal the calling component is rendered in. For a component factored
out of the render closure, which cannot capture the `ModalScope`.

## Theme defaults

| Struct | Field | Type | Description |
|---|---|---|---|
| `ZIndexDefaults` | `modal` | `i32` | The first modal's z-index (`1000`). |
| `ZIndexDefaults` | `modal_step` | `i32` | Added per stacked modal (`10`). |
| `ZIndexDefaults` | `overlay` | `i32` | The backdrop's layer (`300`) - always below a modal. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-modal-z-index` | This modal's computed stacking level, per instance. |
| `--lsx-z-index-modal` | The first modal's z-index, from the theme. |
| `--lsx-z-index-overlay` | The backdrop layer, from the theme. |

## Data attributes

The modal layer writes no state tokens of its own. Its root carries
`data-lsx-scroll-lock`, which is what locks the page behind it.
