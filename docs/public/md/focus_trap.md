# FocusTrap

Crate: `libero`
Import: `use libero::components::FocusTrap;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/accessibility/focus_trap.rs>
Index: [index.md](index.md) lists every other page
Description: Confines Tab and Shift+Tab cycling to its children, for keeping keyboard focus inside an open overlay.

Keeps Tab and Shift+Tab cycling inside its children, as inside an open dialog.
It focuses its first focusable child on mount and adds no box of its own.

## Usage

Tab inside the trap cycles First, Second, Third and Release without reaching
Before or After. Release or Escape switches the trap off, and `restore_focus`
puts focus back where it was before the trap took it.

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, FocusTrap};

#[component]
fn Demo() -> Element {
    let mut trapped = use_signal(|| true);

    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            align: "flex-start",
            Button { variant: "text", "Before" }
            if trapped() {
                FocusTrap {
                    restore_focus: true,
                    Flex {
                        direction: "row",
                        gap: "sm",
                        onkeydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Escape {
                                trapped.set(false);
                            }
                        },
                        Button { variant: "outlined", "First" }
                        Button { variant: "outlined", "Second" }
                        Button { variant: "outlined", "Third" }
                        Button { variant: "filled", onclick: move |_| trapped.set(false), "Release" }
                    }
                }
            } else {
                Button { variant: "outlined", onclick: move |_| trapped.set(true), "Trap focus" }
            }
            Button { variant: "text", "After" }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `children` | `Element` | required | The content that keeps the focus. |
| `restore_focus` | `bool` | `false` | On unmount, hands focus back to whatever held it when the trap mounted. |

Like every component, `FocusTrap` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. `FocusTrapInitialFocus` takes
no props at all.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` or `Shift+Tab` | Cycles through the focusable children, wrapping at either end. |

### Libero handles

- On mount the trap focuses the element marked `data-autofocus`, or else its
  first focusable child.
- With `restore_focus`, unmounting the trap puts focus back where it was before
  the trap took it.

### You must

- To focus nothing visible, so a dialog does not open with its first button
  looking pressed, render `FocusTrapInitialFocus` as the first child.
- Keep a trap only around content that is the one thing that matters on
  screen, such as an open overlay. A keyboard user who cannot Tab out of a
  region has no way back to the page.
- Give the user a way out: the trap has no Escape of its own, so close it on
  Escape and on a button.

```rust
use dioxus::prelude::*;
use libero::components::{Button, FocusTrap, FocusTrapInitialFocus};

#[component]
fn Demo() -> Element {
    rsx! {
        FocusTrap {
            FocusTrapInitialFocus {}
            Button { "Confirm" }
            Button { variant: "text", "Cancel" }
        }
    }
}
```

### Example

A delete confirmation, `FocusTrap { restore_focus: true, .. }` around its
message and two buttons: Tab from the last button wraps to the first, Escape
and Cancel close it, and focus goes back to the button that opened it.

### Limits

- Without `restore_focus`, focus is not restored on unmount: it falls to the
  page body.
- In a desktop WebView or on Android, Tab and Shift+Tab move between the
  children but do not wrap: at either end they leave the trap.

## Theme defaults

None. `FocusTrap` draws nothing to theme.

## CSS variables

None.

## Data attributes

None on the root. Inside the trap, `data-autofocus` on any descendant claims
initial focus; `FocusTrapInitialFocus` sets it on itself.
