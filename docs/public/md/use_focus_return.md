# Focus return

Crate: `libero`
Import: `use libero::hooks::{FocusReturn, use_focus_return};`
Feature: `use-focus-return` (on by default, turns on `use-element`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/focus_return.rs>
Index: [index.md](index.md) lists every other page
Description: Puts focus back on the element that opened a panel or popup once it closes, with fallbacks for a trigger that is gone.

`use_focus_return() -> FocusReturn` puts focus back where it came from once a
panel or popup closes. Without it, focus inside a closing panel drops to the
document body and a keyboard user loses their place. The overlays in libero do
this already; use the hook for a panel of your own. Call `remember_active()` in the handler that opens, on every
open, and `restore()` wherever it closes. A webview, unlike the web and Blitz,
has no focused element to read, so name the trigger with `remember(event)` there.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Checkbox, Flex},
    hooks::use_focus_return,
    sx::sx,
};

#[component]
pub fn Filters() -> Element {
    let mut open = use_signal(|| false);
    let mut in_stock = use_signal(|| false);
    let trigger = use_focus_return();

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Button {
                variant: "outlined",
                aria_expanded: open(),
                aria_controls: "filters-panel",
                onclick: move |_| {
                    if !open() {
                        trigger.remember_active();
                    }
                    open.toggle();
                },
                "Filters"
            }
            if open() {
                Flex {
                    id: "filters-panel",
                    direction: "column",
                    align: "flex-start",
                    gap: "sm",
                    sx: sx().padding("md").background("muted.1").border_radius("8px"),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Named(NamedKey::Escape) {
                            open.set(false);
                            trigger.restore();
                        }
                    },
                    Checkbox {
                        label: "In stock only",
                        checked: in_stock(),
                        onchange: move |next| in_stock.set(next),
                    }
                    Button {
                        onclick: move |_| {
                            open.set(false);
                            trigger.restore();
                        },
                        "Apply"
                    }
                }
            }
        }
    }
}
```

## API

```rust,ignore
pub fn use_focus_return() -> FocusReturn
```

| Method | Returns | Description |
|---|---|---|
| `remember_active()` | `()` | Remembers whatever has focus now, read from the document: the web and Blitz have one, a webview does not and remembers nothing. Call it synchronously in the handler that opens, on every open. |
| `remember(event: Event<MountedData>)` | `()` | Names the trigger instead of reading the focused element, for a webview. Use as the trigger's `onmounted`. Arm it once; `restore()` keeps it. |
| `fallback(element: ElementHandle)` | `()` | Where focus goes if the remembered element is gone, such as the list a deleted row lived in. Appends, so call it once per tier, nearest first. Naming an element twice does nothing. |
| `restore()` | `()` | Call it wherever the panel closes. Puts focus back on the trigger, or on the first fallback still in the document. Consumes a `remember_active()` snapshot, so arm it again on every open. |

`FocusReturn` is `Copy`. `restore()` focuses in a spawned task, after the
closing event has finished, so it is safe to call from that event's handler.

## Accessibility

### Libero handles

- `restore()` puts focus back on the remembered element. In the demo, Tab into
  the panel, then press Apply or Escape, and focus lands on Filters again.
  [Collapse](collapse.md) shows the same return on an animated panel.

### Example

A filters panel of your own: the Filters button calls `remember_active()` as
it opens the panel, and Apply calls `restore()`, so a keyboard user lands back
on Filters instead of the top of the page.

### Limits

- Some browsers do not focus a button on a mouse click, so a panel opened
  with the mouse may remember the body. That only matters to a keyboard user,
  and for them the trigger has focus.
