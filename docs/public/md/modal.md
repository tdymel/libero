# Modal

Crate: `libero`
Import: `use libero::components::{Modal, Dialog};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/modal.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A focus-trapped, dimmed, scroll-locking layer that is mounted only while open; pair it with `Dialog` for the dialog role and aria-modal semantics.

A focus-trapped, dimmed layer that locks scroll. It takes no props beyond
`onclose` and its children - it is mounted only while open, so the caller's own
state is the switch, and it has no opinion on content: pair it with `Dialog` for
the role and aria-modal semantics.

Escape and a backdrop click only *request* a close, through `onclose`. Nothing
closes unless the caller's state says so, so a confirm-before-close flow needs no
extra API.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Dialog, Modal, Text, Title};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button { variant: "outlined", onclick: move |_| open.set(true), "Open modal" }
        if open() {
            Modal {
                onclose: move |_| open.set(false),
                Dialog {
                    aria_label: "Example modal",
                    Title { size: "lg", "Example modal" }
                    Text { "Closes on Escape or by clicking the backdrop." }
                    Button { variant: "outlined", onclick: move |_| open.set(false), "Close" }
                }
            }
        }
    }
}
```

A descendant that is not holding the state can still ask for a close through
`use_modal_context()`, whose `close()` calls the same `onclose` the modal was
given:

```rust
use dioxus::prelude::*;
use libero::{components::Button, hooks::use_modal_context};

#[component]
fn CloseButton() -> Element {
    let modal = use_modal_context();

    rsx! {
        Button { variant: "text", onclick: move |_| modal.close(), "Cancel" }
    }
}
```

## Accessibility

`Modal` itself carries no role - it is the dimmed, focus-trapped layer. `Dialog`
supplies `role="dialog"`, and adds `aria-modal="true"` on its own once it detects
a `Modal` ancestor, so nesting the two is all that is needed. Give the dialog an
`aria_label` (or your own `aria-labelledby`) or it is announced unnamed.

Focus is trapped inside the modal for as long as it is mounted, Escape fires
`onclose`, and the page behind it is scroll-locked
(`data-lsx-scroll-lock`). Stacked modals each take their own z-index -
`z_index.modal + n * z_index.modal_step` - so a modal opened from a modal sits
above it. Because it is mounted conditionally, closing it removes the trap and
returns focus to normal document order.

## Props

### Modal

| Prop | Type | Default | Description |
|---|---|---|---|
| `onclose` | `EventHandler<()>` | - | Called on Escape or a backdrop click - the caller's state, not `Modal`, decides whether it actually closes. |
| `children` | `Element` | required | Content, mounted only while open. Pair with `Dialog` for the role and aria-modal semantics. |

### Dialog

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | - | Accessible name for the dialog. |
| `radius` | `ThemeAwareValue` | `md` | Corner radius - the radius scale, or any CSS length. |
| `size` | `ThemeAwareValue` | `md` | Caps the dialog's width. |
| `variables` | `Variables` | - | Layered onto `Dialog`'s own - e.g. `Drawer`'s anchor/size vars. |
| `children` | `Element` | required | The dialog's content. |

Like every component, both also take the shared props `sx`, `class`, `states`,
and any extra HTML attributes.

## Theme defaults

| Struct | Field | Type | Description |
|---|---|---|---|
| `DialogDefaults` | `size` | `Sizes<u16>` | Max width in px per size step - `240, 300, 510, 600, 750, 900`. |
| `ZIndexDefaults` | `modal` | `i32` | The first modal's z-index (`1000`). |
| `ZIndexDefaults` | `modal_step` | `i32` | Added per stacked modal (`10`). |
| `ZIndexDefaults` | `overlay` | `i32` | The backdrop's layer (`300`) - always below a modal. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-modal-z-index` | This modal's computed stacking level, per instance. |
| `--lsx-z-index-modal` | The first modal's z-index, from the theme. |
| `--lsx-z-index-overlay` | The backdrop layer, from the theme. |
| `--lsx-dialog-size-<size>` | Dialog max width for that size step. |
| `--lsx-dialog-size-override` | Set by `Dialog`'s `size` prop. |
| `--lsx-dialog-radius` | Set by `Dialog`'s `radius` prop; falls back to `--lsx-radius-md`. |

## Data attributes

`Modal` writes no state tokens of its own. Its root carries
`data-lsx-scroll-lock`, which is what locks the page behind it; `Dialog` renders
`role="dialog"` plus `aria-modal="true"` inside a modal.
