# Modal

Crate: `libero`
Import: `use libero::hooks::{ModalHandle, ModalScope, use_modal};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/use_modal.rs>
Index: [index.md](index.md) lists every other page
Description: A hook that opens a render closure as a modal, with arguments and a result per opening.

A modal is a hook, not a component. `use_modal` takes a render closure and
returns a handle that opens it. The content is built only while it shows. Each
opening passes its arguments to the closure's `ModalScope`, and settles with
what `resolve` answered, or `None` when dismissed.

Make the answer the dialog's own enum, not a `bool`. The handle is `Copy`, so a
trigger elsewhere in the tree can take it as a prop or from context. A
[`Dialog`](dialog.md) inside closes the modal from its own close button.

## Usage

Wrap `use_modal` in a hook of your own. Its parameters are shared by every
opening, and the `ModalScope` carries the arguments of the one showing.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, Text},
    hooks::{ModalHandle, ModalScope, use_modal},
};

/// A dismissal answers nothing, so the caller matches on `Option<SaveChoice>`.
#[derive(Clone, Copy, PartialEq)]
enum SaveChoice {
    Save,
    Discard,
}

/// `discard_label` is shared by every opening.
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
    let mut answer = use_signal(|| "none yet");

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| {
                prompt.open_with("notes.md").onresult(move |result| {
                    answer.set(match result {
                        Some(SaveChoice::Save) => "saved",
                        Some(SaveChoice::Discard) => "discarded",
                        None => "dismissed",
                    });
                });
            },
            "Close editor"
        }
        Text { "Last answer: {answer}" }
    }
}
```

Content in the render closure captures the `ModalScope`. A component outside
the closure closes the modal with `use_modal_close()`:

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

`ModalContext` tells a component whether it is in a modal at all:

```rust
use dioxus::prelude::*;
use libero::{components::Button, context::ModalContext};

#[component]
fn CloseIfModal() -> Element {
    let Some(modal) = try_use_context::<ModalContext>().filter(ModalContext::is_modal) else {
        return rsx! {};
    };

    rsx! {
        Button { variant: "text", onclick: move |_| modal.close(), "Close" }
    }
}
```

## API

### `use_modal`

```rust,ignore
pub fn use_modal<S: Clone + 'static, R: Clone + 'static>(
    render: impl FnMut(ModalScope<S, R>) -> Element + 'static,
) -> ModalHandle<S, R>
```

| Argument | Type | Default | Description |
|---|---|---|---|
| `render` | `impl FnMut(ModalScope<S, R>) -> Element` | required | Builds the content, usually a `Dialog`, while the modal is open. Style it there: the `Dialog`'s own `sx` and `parts` reach it in the portal. Returns a `ModalHandle<S, R>`. `R` defaults to `()`, for a modal that answers nothing. Call it under `LiberoProvider`, in a component that outlives every trigger. The modal unmounts with that component, and its opening settles as dismissed. |

### `ModalHandle<S, R = ()>`

| Method | Returns | Description |
|---|---|---|
| `open_with(args: impl Into<S>)` | `Opening<R>` | Opens with these arguments, replacing whatever was showing. |
| `open()` | `Opening<R>` | Opens with `S::default()`. Needs `S: Default`. |
| `close()` | `()` | Dismisses whatever this modal is showing. |
| `is_open()` | `bool` | Whether this modal is showing. |

### `ModalScope<S, R = ()>`

| Method | Returns | Description |
|---|---|---|
| `args()` | `S` | The arguments this opening was given. |
| `close()` | `()` | Ends it as a dismissal, the same as Escape. |
| `resolve(value: R)` | `()` | Ends it with an answer for the caller. |

### `Opening<R = ()>`

| Method | Returns | Description |
|---|---|---|
| `onresult(f: impl FnMut(Option<R>))` | `Self` | Runs when this opening settles, with `None` if it was dismissed. A replaced opening never runs it. Awaiting the `Opening` gives the same answer instead, and `None` once replaced. |
| `close()` | `()` | Closes this opening, if it is still the one showing. |
| `.await` | `Option<R>` | The same outcome, as a future. `None` once replaced. |

### `ModalContext`

| Method | Returns | Description |
|---|---|---|
| `is_modal()` | `bool` | Whether the content is in a modal. |
| `close()` | `()` | Dismisses the modal around the content. Does nothing outside one. |

A floating window provides an empty `ModalContext`, so its content is not
modal.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Escape` | Dismisses the modal, as a backdrop click does. |
| `Tab` or `Shift+Tab` | Moves the focus within the modal. It does not leave while the modal shows. |

### Libero handles

- Focus moves into the modal, and back to the trigger once it closes.
- The focus trap, Escape and backdrop dismissal come from the modal. A `Dialog`
  on its own has none of them.
- A dismissal settles the `Opening` with `None`, so a handler written for an
  answer never runs on it.
- Android's Back button dismisses the top modal, as Escape does, rather than
  closing the app.

### You must

- Name the `Dialog` with its `title`, or `aria_label`.

### Limits

- On Android, a modal opened without a tap (on mount or from a timer) may let
  Back close the app.

## Theme defaults

| Struct | Field | Type | Description |
|---|---|---|---|
| `ZIndexDefaults` | `modal` | `i32` | The first modal's z-index (`1000`). |
| `ZIndexDefaults` | `modal_step` | `i32` | Added per stacked modal (`10`). |
| `ZIndexDefaults` | `overlay` | `i32` | The backdrop's layer (`300`), always below a modal. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-modal-z-index` | This modal's stacking level. |
| `--lsx-z-index-modal` | The first modal's z-index, from the theme. |
| `--lsx-z-index-overlay` | The backdrop layer, from the theme. |

## Data attributes

The modal's root carries `data-lsx-scroll-lock`. The page behind it does not
scroll while the modal is open. A classic scrollbar's width is added to the
`html` element's own `padding-right`, so the page does not shift and the backdrop
covers the full width. Your own `position: fixed` element with `right: 0` still
shifts by that width.
