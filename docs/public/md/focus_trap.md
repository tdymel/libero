# FocusTrap

Crate: `libero`
Import: `use libero::components::FocusTrap;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/accessibility/focus_trap.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Confines Tab and Shift+Tab cycling to its children, for keeping keyboard focus inside an open overlay.

Confines Tab/Shift+Tab cycling to its children - the same mechanism
the [modal](modal.md) layer uses internally to keep keyboard focus inside an open dialog.
It focuses its first focusable child on mount, so mounting a trap moves focus
into it; Tab from there cycles the children without ever reaching a control
outside.

## Usage

The trap needs focusable siblings outside it, or "focus cannot leave" has
nothing to fail against. Tab inside the trap cycles First/Second/Third and never
reaches Before or After.

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

The root is `display: contents`, so the trap adds no box and no layout of its
own - remove it and the children sit exactly where they sat.

## Accessibility

On mount the trap focuses the element marked `data-autofocus`, or its first
focusable descendant otherwise. To land initial focus on nothing visible - so a
dialog does not open with its first button looking pressed - render
`FocusTrapInitialFocus` as the first child.

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

A trap is only correct while its content is the only thing on screen that
matters. Do not leave one mounted around ordinary page content: a keyboard user
who cannot Tab out of a region has no way back to the rest of the page.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `children` | `Element` | required | The content Tab/Shift+Tab cycling is confined to. |

Like every component, `FocusTrap` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. `FocusTrapInitialFocus` takes
no props at all.

## Theme defaults

None - `FocusTrap` has no visual surface to theme.

## CSS variables

None.

## Data attributes

None on the root. Inside the trap, `data-autofocus` on any descendant claims
initial focus; `FocusTrapInitialFocus` sets it on itself.
