# FocusTrap

Crate: `libero`
Import: `use libero::components::FocusTrap;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/accessibility/focus_trap.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Confines Tab and Shift+Tab cycling to its children, for keeping keyboard focus inside an open overlay.

Keeps Tab and Shift+Tab cycling inside its children, as inside an open dialog.
It focuses its first focusable child on mount and adds no box of its own.

## Usage

Tab inside the trap cycles First, Second and Third without reaching Before or
After.

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, FocusTrap};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            align: "flex-start",
            Button { variant: "text", "Before" }
            FocusTrap {
                Flex {
                    direction: "row",
                    gap: "sm",
                    Button { variant: "outlined", "First" }
                    Button { variant: "outlined", "Second" }
                    Button { variant: "outlined", "Third" }
                }
            }
            Button { variant: "text", "After" }
        }
    }
}
```

## Accessibility

On mount the trap focuses the element marked `data-autofocus`, or else its
first focusable child. To focus nothing visible, so a dialog does not open with
its first button looking pressed, render `FocusTrapInitialFocus` as the first
child.

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

Keep a trap only around content that is the one thing that matters on screen,
such as an open overlay. A keyboard user who cannot Tab out of a region has no
way back to the page.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `children` | `Element` | required | The content that keeps the focus. |

Like every component, `FocusTrap` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. `FocusTrapInitialFocus` takes
no props at all.

## Theme defaults

None. `FocusTrap` draws nothing to theme.

## CSS variables

None.

## Data attributes

None on the root. Inside the trap, `data-autofocus` on any descendant claims
initial focus; `FocusTrapInitialFocus` sets it on itself.
