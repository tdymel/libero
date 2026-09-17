# use_popover

Crate: `libero`
Import: `use libero::hooks::use_popover;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/popover/mod.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Portals a box to the document root and places it next to an anchor, flipping and shifting to stay on screen.

`use_popover(anchor, open, options) -> PopoverHandle` portals a box to the
document root and places it next to an anchor, flipping and shifting to stay on
screen. It owns no open state and adds no semantics. [Popover](popover.md) has
the full story.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Button},
    hooks::{Align, PopoverOptions, Side, use_element, use_id, use_popover, use_theme},
    platform::ElementApi,
    sx::sx,
};

#[component]
fn ShippingInfo() -> Element {
    let theme = use_theme();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    let box_id = use_id();

    let popover = use_popover(
        anchor,
        opened(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Bottom)
            .align(Align::Start),
    );
    let floating = *popover.floating();

    // A dialog takes focus once placed, once per opening.
    let mut entered = use_signal(|| false);
    use_effect(move || match (opened(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            let _ = floating.focus();
        }
        (false, _) => entered.set(false),
        _ => {}
    });
    // Escape and Tab close it and hand focus back to the trigger.
    let mut close = move |event: &KeyboardEvent| {
        opened.set(false);
        let _ = anchor.focus();
        if event.key() == Key::Escape || event.modifiers().shift() {
            event.prevent_default();
        }
    };

    popover.show(opened().then(|| rsx! {
        Box {
            id: "{box_id}",
            role: "dialog",
            aria_label: "Shipping",
            tabindex: "-1",
            onkeydown: move |event: KeyboardEvent| {
                if matches!(event.key(), Key::Escape | Key::Tab) {
                    close(&event);
                }
            },
            style: popover.style(),
            onmounted: floating.mount(),
            sx: sx().background("surface").padding("var(--lsx-popover-padding)"),
            "Two to four working days."
        }
    }));

    rsx! {
        Button {
            onmounted: anchor.mount(),
            onclick: move |_| opened.toggle(),
            // Focus is still here before the box is placed.
            onkeydown: move |event: KeyboardEvent| match event.key() {
                Key::Escape if opened() => {
                    event.prevent_default();
                    opened.set(false);
                }
                Key::Tab if opened() => opened.set(false),
                _ => {}
            },
            aria_haspopup: "dialog",
            aria_expanded: "{opened()}",
            aria_controls: "{box_id}",
            "Shipping"
        }
    }
}
```

Most of the code is the keyboard, because the hook leaves it to you.
`show(None)` is how a closed popover stops rendering.

## Accessibility

Give the box its role and the trigger `aria-haspopup`, `aria-expanded` and
`aria-controls`. Drive `aria-expanded` from the same signal you pass the hook.
Escape must close the box, and focus is not trapped. Tab closes it and moves
on.

## Web and native

Natively an open popover drifts when the page scrolls, because only the web
reports document scrolls. The box is portaled, so listen for Escape on the
trigger as well, as above.

## API

```rust,ignore
pub fn use_popover(anchor: ElementHandle, open: bool, options: PopoverOptions) -> PopoverHandle
```

| `PopoverHandle` method | Returns | Description |
|---|---|---|
| `floating()` | `&ElementHandle` | The handle to mount on the box, so it can be measured. |
| `placed()` | `bool` | Whether the box has been measured. |
| `placement()` | `Placement` | The side and align it landed on, after flipping. |
| `style()` | `Option<String>` | The box's `style`: fixed position, coordinates and width. |
| `show(content: Option<Element>)` | `()` | Portals the box. `None` removes it. |

`PopoverOptions::new(gap, padding)` takes the theme's two defaults. Its
builders are `side`, `align`, `flip`, `shift`, `width` and `remeasure`.
